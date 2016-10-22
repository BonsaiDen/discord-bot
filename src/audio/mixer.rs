// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::cmp;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// External Dependencies ------------------------------------------------------
use rand::{thread_rng, Rng};


// Internal Dependencies ------------------------------------------------------
use super::MixerList;
use ::effect::Effect;


// Statics --------------------------------------------------------------------
static MAX_PARALLEL_SOURCES: usize = 2;
static MIXER_DELAY_MILLIS: u64 = 500;


// Mixer Queue  ---------------------------------------------------------------
pub enum MixerCommand {
    PlayEffects(Vec<Effect>),
    QueueEffects(Vec<Effect>),
    ClearQueue
}

pub type MixerQueue = Arc<Mutex<VecDeque<MixerCommand>>>;

pub struct EmptyMixerQueue;

impl EmptyMixerQueue {
    pub fn create() -> MixerQueue {
        Arc::new(Mutex::new(VecDeque::new()))
    }
}


// Audio Playback Mixer Abstraction -------------------------------------------
pub struct Mixer {
    id: u64,
    command_queue: MixerQueue,
    audio_buffer: [i16; 960 * 2],
    active_sources: Vec<MixerList>,
    queued_sources: VecDeque<MixerList>,
    delay: u64
}


// Public Interface -----------------------------------------------------------
impl Mixer {

    pub fn new(command_queue: MixerQueue) -> Mixer {

        let mut rng = thread_rng();
        let mixer = Mixer {
            command_queue: command_queue.clone(),
            audio_buffer: [0; 960 * 2],
            active_sources: Vec::new(),
            queued_sources: VecDeque::new(),
            delay: MIXER_DELAY_MILLIS,
            id: rng.next_u64()
        };

        info!("{} Created", mixer);

        mixer

    }

    fn update_sources(&mut self) {

        if self.active_sources.len() < MAX_PARALLEL_SOURCES {

            // Pop the next command from the queue
            if let Some(command) = {
                self.command_queue.lock().expect("No command queue lock.").pop_front()

            } {
                match command {
                    MixerCommand::PlayEffects(effects) => {
                        info!("{} Playing effects list...", self);
                        self.active_sources.push(MixerList::new(effects));
                    },
                    MixerCommand::QueueEffects(effects) => {
                        info!("{} Queueing effects list...", self);
                        self.queued_sources.push_back(MixerList::new(effects));
                    },
                    MixerCommand::ClearQueue => {
                        info!("{} List queues cleared", self);
                        self.active_sources.clear();
                        self.queued_sources.clear();
                    }
                }

            // If there is no next command in the queue and we currently have no
            // active sources, pop a source from the queued stack and make it active
            } else if self.active_sources.is_empty() {
                if let Some(source) = self.queued_sources.pop_front() {
                    self.active_sources.push(source);
                }
            }

        } else if let Some(&MixerCommand::ClearQueue) = {
            self.command_queue.lock().expect("No command queue lock.").front()
        } {
            info!("{} List queues cleared", self);
            self.command_queue.lock().expect("No command queue lock.").pop_front();
            self.active_sources.clear();
            self.queued_sources.clear();
        }

    }

    fn mix(&mut self, buffer: &mut [i16]) -> usize {
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
        for list in &mut self.active_sources {

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

            list.update();

        }

        // Remove inactive sources once they are done
        self.active_sources.retain(|list| list.is_active());

        mixed

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

