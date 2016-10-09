// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;


// Statics --------------------------------------------------------------------
static MAX_PARALLEL_SOURCES: usize = 2;
static MIXER_DELAY_MILLIS: u64 = 300;


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

struct MixerSource;


// Audio Playback Mixer Abstraction -------------------------------------------
pub struct Mixer {
    command_queue: MixerQueue,
    audio_buffer: [i16; 960 * 2],
    active_sources: Vec<MixerSource>,
    queued_sources: VecDeque<MixerSource>,
    delay: u64
}


// Public Interface -----------------------------------------------------------
impl Mixer {

    pub fn new(command_queue: MixerQueue) -> Mixer {

        let mixer = Mixer {
            command_queue: command_queue.clone(),
            audio_buffer: [0; 960 * 2],
            active_sources: Vec::new(),
            queued_sources: VecDeque::new(),
            delay: MIXER_DELAY_MILLIS
        };

        info!("{} Created", mixer);

        mixer

    }

    fn update_sources(&mut self) {

        if self.active_sources.len() < MAX_PARALLEL_SOURCES {

            // Pop the next command from the queue
            if let Some(command) = {
                self.command_queue.lock().unwrap().pop_front()

            } {
                match command {
                    MixerCommand::PlayEffects(_) => {
                        info!("{} Playing source", self);
                        self.active_sources.push(MixerSource);
                    },
                    MixerCommand::QueueEffects(_) => {
                        info!("{} Queued source", self);
                        self.queued_sources.push_back(MixerSource);
                    },
                    MixerCommand::ClearQueue => {
                        info!("{} Source queue cleared", self);
                        self.active_sources.clear();
                        self.queued_sources.clear();
                    }
                }

            // If there is no next comman in the queue and we currently have no active
            // sources, pop a source from the queued stack and make it active
            } else if self.active_sources.is_empty() {
                if let Some(source) = self.queued_sources.pop_front() {
                    self.active_sources.push(source);
                }
            }

        } else if let Some(&MixerCommand::ClearQueue) = {
            self.command_queue.lock().unwrap().front()
        } {
            info!("{} Source queue cleared", self);
            self.command_queue.lock().unwrap().pop_front();
            self.active_sources.clear();
            self.queued_sources.clear();
        }

    }

    fn mix(&mut self, _: &mut [i16]) -> usize {
        if self.delay == 0 {
            0

        } else {
            self.delay -= 20;
            0
        }

    }

}


// Audio Source Implementation ------------------------------------------------
impl AudioSource for Mixer {

    fn is_stereo(&mut self) -> bool {
        true
    }

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        self.update_sources();
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
        write!(f, "[AudioMixer]")
    }
}

