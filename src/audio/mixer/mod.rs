// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::cmp;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// External Dependencies ------------------------------------------------------
use rand::{thread_rng, Rng};


// Modules --------------------------------------------------------------------
mod list;
mod source;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use self::list::MixerList;


// Re-Exports -----------------------------------------------------------------
pub use self::source::MixerSource;


// Statics --------------------------------------------------------------------
static MAX_PARALLEL_SOURCES: usize = 2;
static MIXER_DELAY_MILLIS: u64 = 5000;
lazy_static! {
    static ref EFFECT_PLAYBACK_ID: AtomicUsize = AtomicUsize::new(0);
}


// Mixer Commands -------------------------------------------------------------
pub enum MixerCommand {
    PlayEffects(Vec<(Effect, usize)>),
    QueueEffects(Vec<(Effect, usize)>),
    ClearDelay,
    ClearQueue
}


// Mixer Events ---------------------------------------------------------------
#[derive(Debug)]
pub enum MixerEvent {
    Completed(Effect, usize),
    Canceled(Effect, usize)
}


// Audio Playback Mixer Abstraction -------------------------------------------
pub struct Mixer {
    id: u64,
    command_queue: Receiver<MixerCommand>,
    command_buffer: VecDeque<MixerCommand>,
    event_queue: Sender<MixerEvent>,
    active_source_lists: Vec<MixerList>,
    queued_source_lists: VecDeque<MixerList>,
    audio_buffer: [i16; 960 * 2],
    delay: u64
}


// Public Interface -----------------------------------------------------------
impl Mixer {

    pub fn new(
        command_queue: Receiver<MixerCommand>,
        event_queue: Sender<MixerEvent>

    ) -> Mixer {

        let mut rng = thread_rng();
        let mixer = Mixer {
            command_queue: command_queue,
            command_buffer: VecDeque::new(),
            event_queue: event_queue,
            active_source_lists: Vec::new(),
            queued_source_lists: VecDeque::new(),
            audio_buffer: [0; 960 * 2],
            delay: MIXER_DELAY_MILLIS,
            id: rng.next_u64()
        };

        info!("{} Created", mixer);

        mixer

    }

    pub fn next_effect_id() -> usize {
        EFFECT_PLAYBACK_ID.fetch_add(1, Ordering::SeqCst)
    }

}


// Internal Interface ---------------------------------------------------------
impl Mixer {

    fn update_sources(&mut self) {

        if self.active_source_lists.len() < MAX_PARALLEL_SOURCES {

            // Pop the next available command from the queue
            if let Some(command) = self.command_buffer.pop_front() {
                match command {
                    MixerCommand::PlayEffects(effects) => {
                        info!("{} Playing effects list...", self);
                        self.active_source_lists.push(MixerList::new(effects));
                    },
                    MixerCommand::QueueEffects(effects) => {
                        info!("{} Queueing effects list...", self);
                        self.queued_source_lists.push_back(MixerList::new(effects));
                    },
                    MixerCommand::ClearQueue => self.clear(),
                    _ => unreachable!()
                }

            // If there is no next command in the queue and we currently have no
            // active sources, pop a source from the queued stack and make it active
            } else if self.active_source_lists.is_empty() {
                if let Some(source) = self.queued_source_lists.pop_front() {
                    self.active_source_lists.push(source);
                }
            }

        }

    }

    fn mix(&mut self, buffer: &mut [i16]) -> usize {

        // Pull commands from receiver
        while let Ok(command) = self.command_queue.try_recv() {
            match command {

                // Clear audio delay once we joined the channel
                MixerCommand::ClearDelay => {
                    info!("{} Delay cleared", self);
                    self.delay = 0;
                },

                // Always clear queue if requested
                MixerCommand::ClearQueue => self.clear(),

                // Push other commands into the buffer
                _ => self.command_buffer.push_back(command)

            }
        }

        if self.delay == 0 {
            self.update_sources();
            self.mix_sources(buffer)

        } else {
            self.delay -= 20;
            0
        }

    }

    fn mix_sources(&mut self, buffer: &mut [i16]) -> usize {

        // Maximum possible sample value
        let max_sample_value = i16::max_value() as f32;

        // Clear buffer
        let samples = buffer.len();
        for item in buffer.iter_mut().take(samples) {
            *item = 0;
        }

        // Mix Samples from all active sources into the buffer
        let mut mixed = 0;
        for list in &mut self.active_source_lists {

            if let Some(source) = list.get_active_source() {

                let channels = source.channels();
                let channel_offset = 3 - channels;
                if let Some(written) = source.read_frame(
                    &mut self.audio_buffer[..960 * channels]
                ) {

                    mixed = cmp::max(written * channel_offset, mixed);

                    for e in 0..written {

                        // Double interval when mixing mono
                        let i = e * channel_offset;

                        // Left / Mono Sample
                        let s = buffer[i] as f32 + self.audio_buffer[e] as f32;
                        buffer[i] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;

                        // Right Sample
                        if channels == 1 {
                            let s = buffer[i + 1] as f32 + self.audio_buffer[e] as f32;
                            buffer[i + 1] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;
                        }

                    }

                }

            }

            // Update the list to play the next effect and return any previously
            // finished effect
            if let Some(effect) = list.udpate_and_complete() {
                self.event_queue.send(
                    MixerEvent::Completed(effect.0, effect.1)

                ).ok();
            }

        }

        // Remove inactive sources once they have completed playing
        self.active_source_lists.retain(|list| list.is_active());

        mixed

    }

    fn clear(&mut self) {

        info!("{} Clearing list queues...", self);

        for mut list in self.active_source_lists.drain(0..) {
            for effect in list.clear() {
                self.event_queue.send(
                    MixerEvent::Canceled(effect.0, effect.1)

                ).ok();
            }
        }

        for mut list in self.queued_source_lists.drain(0..) {
            for effect in list.clear() {
                self.event_queue.send(
                    MixerEvent::Canceled(effect.0, effect.1)

                ).ok();
            }
        }

    }

}


// Audio Source Implementation ------------------------------------------------
impl AudioSource for Mixer {

    fn is_stereo(&mut self) -> bool {
        true
    }

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        Some(self.mix(buffer))
    }

}


// Traits ---------------------------------------------------------------------
impl Drop for Mixer {
    fn drop(&mut self) {
        info!("{} Dropped", self);
    }
}

impl fmt::Display for Mixer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[AudioMixer {}]", self.id)
    }
}


// Helpers --------------------------------------------------------------------
pub fn compress(x: f32, threshold: f32) -> f32 {

    // threshold    alpha
    // ------------------
    // 0            2.51
    // 0.05         2.67
    // 0.1          2.84
    // 0.15         3.04
    // 0.2          3.26
    // 0.25         3.52
    // 0.3          3.82
    // 0.35         4.17
    // 0.4          4.59
    // 0.45         5.09
    // 0.5          5.71
    // 0.55         6.49
    // 0.6          7.48
    // 0.65         8.81
    // 0.7          10.63
    // 0.75         13.3
    // 0.8          17.51
    // 0.85         24.97
    // 0.9          41.15
    // 0.95         96.09
    if x >= -threshold && x <= threshold {
        x

    } else {
        let alpha = 7.48; // for threshold=0.6
        let xa = x.abs();
        let a = (1.0 + alpha * ((xa - threshold) / (2.0 - threshold))).ln();
        let b = (1.0 + alpha).ln();
        (x / xa) * (threshold + (1.0 - threshold) * (a / b))
    }

}

