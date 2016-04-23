// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// Internal Dependencies ------------------------------------------------------
use super::queue::Queue;
use super::source::SourceList;


// Voice Audio Mixer ----------------------------------------------------------
pub struct Mixer {
    queue: Queue,
    source_buffer: [i16; 960 * 2],
    active_lists: Vec<SourceList>,
    waiting_lists: Vec<SourceList>
}

impl Mixer {

    pub fn new(queue: Queue) -> Mixer {
        Mixer {
            queue: queue,
            active_lists: Vec::new(),
            waiting_lists: Vec::new(),
            source_buffer: [0; 960 * 2]
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

