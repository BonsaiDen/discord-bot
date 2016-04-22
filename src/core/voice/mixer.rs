// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// Internal Dependencies ------------------------------------------------------
use super::queue::Queue;


// Voice Audio Mixer ----------------------------------------------------------
pub struct Mixer {
    queue: Queue
}

impl Mixer {

    pub fn new(queue: Queue) -> Mixer {
        Mixer {
            queue: queue
        }
    }

    fn mix(&mut self, buffer: &mut [i16]) -> usize {
        0
    }

}


// Traits ---------------------------------------------------------------------
impl AudioSource for Mixer {

    fn is_stereo(&mut self) -> bool {
        true
    }

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        Some(self.mix(buffer))
    }

}

