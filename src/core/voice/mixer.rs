// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::collections::VecDeque;


// Statics --------------------------------------------------------------------
static MAX_PARALLEL_SOURCE_LISTS: usize = 2;


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// Internal Dependencies ------------------------------------------------------
use super::queue::{Queue, QueueEntry};
use super::source::SourceList;


// Voice Audio Mixer ----------------------------------------------------------
pub struct Mixer {
    queue: Queue,
    source_buffer: [i16; 960 * 2],
    active_lists: Vec<SourceList>,
    waiting_lists: VecDeque<SourceList>
}

impl Mixer {

    pub fn new(queue: Queue) -> Mixer {
        Mixer {
            queue: queue,
            active_lists: Vec::new(),
            waiting_lists: VecDeque::new(),
            source_buffer: [0; 960 * 2]
        }
    }

    fn mix(&mut self, buffer: &mut [i16]) -> usize {

        // Allow maximum of two sound lists to be played in parallel
        if self.active_lists.len() < MAX_PARALLEL_SOURCE_LISTS {

            // Pop the next effect list from the queue
            if let Some(entry) = {
                self.queue.lock().unwrap().pop_front()

            } {
                match entry {
                    QueueEntry::EffectList(effects, delay) => {
                        self.active_lists.push(SourceList::new(effects, delay));
                    },
                    QueueEntry::QueuedEffectList(effects, delay) => {
                        self.waiting_lists.push_back(SourceList::new(effects, delay));
                    },
                    QueueEntry::Reset => {
                        self.active_lists.clear();
                        self.waiting_lists.clear();
                    }
                }

            // If there is no next effect in the queue and we currently have no active
            // lists, pop a list from the waiting stack and make it active
            } else if self.active_lists.is_empty() {
                if let Some(list) = self.waiting_lists.pop_front() {
                    self.active_lists.push(list);
                }
            }

        }

        // Maximum possible sample value
        let max_sample_value = i16::max_value() as f32;

        // Clear buffer
        let samples = buffer.len();
        for i in 0..samples {
            buffer[i] = 0;
        }

        // Mix Samples from all active sources into the buffer
        let mut mixed = 0;
        for list in &mut self.active_lists {

            if let Some(source) = list.get_active_source() {

                let channels = source.channels();
                if let Some(written) = source.read_frame(
                    &mut self.source_buffer[..960 * channels]
                ) {

                    // Mix Stereo
                    if channels == 2 {
                        mixed = cmp::max(written, mixed);
                        for i in 0..written {
                            let s = buffer[i] as f32 + self.source_buffer[i] as f32;
                            buffer[i] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;
                        }

                    // Mix Mono
                    } else {
                        mixed = cmp::max(written * 2, mixed);
                        for e in 0..written {

                            let i = e * 2;

                            // Left Sample
                            let s = buffer[i] as f32 + self.source_buffer[e] as f32;
                            buffer[i] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;

                            // Right Sample
                            let s = buffer[i + 1] as f32 + self.source_buffer[e] as f32;
                            buffer[i + 1] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;

                        }
                    }

                }

            }

            list.update();

        }

        // Remove inactive sources once they are done
        self.active_lists.retain(|list| list.is_active());

        mixed

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


// Helpers --------------------------------------------------------------------
fn compress(x: f32, threshold: f32) -> f32 {

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

