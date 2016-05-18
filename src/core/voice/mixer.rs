// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};


// Statics --------------------------------------------------------------------
static MAX_PARALLEL_SOURCE_LISTS: usize = 2;


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioSource;


// Internal Dependencies ------------------------------------------------------
use super::queue::{Queue, QueueEntry};
use super::source::SourceList;
use super::util::compress;


// Types ----------------------------------------------------------------------
pub type MixerCount = Arc<AtomicUsize>;

pub struct EmptyMixerCount;

impl EmptyMixerCount {
    pub fn create() -> MixerCount {
        Arc::new(AtomicUsize::new(0))
    }
}


// Voice Audio Mixer ----------------------------------------------------------
pub struct Mixer {
    queue: Queue,
    mixer_count: MixerCount,
    source_buffer: [i16; 960 * 2],
    active_lists: Vec<SourceList>,
    waiting_lists: VecDeque<SourceList>
}

impl Mixer {

    pub fn new(queue: Queue, mixer_count: MixerCount) -> Mixer {
        mixer_count.fetch_add(1, Ordering::SeqCst);
        Mixer {
            queue: queue,
            mixer_count: mixer_count,
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
        for item in buffer.iter_mut().take(samples) {
            *item = 0;
        }

        // Mix Samples from all active sources into the buffer
        let mut mixed = 0;
        for list in &mut self.active_lists {

            if let Some(source) = list.get_active_source() {

                let channels = source.channels();
                let channel_offset = 3 - channels;
                if let Some(written) = source.read_frame(
                    &mut self.source_buffer[..960 * channels]
                ) {

                    mixed = cmp::max(written * channel_offset, mixed);

                    for e in 0..written {

                        // Double interval when mixing mono
                        let i = e * channel_offset;

                        // Left / Mono Sample
                        let s = buffer[i] as f32 + self.source_buffer[e] as f32;
                        buffer[i] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;

                        // Right Sample
                        if channels == 1 {
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

impl Drop for Mixer {
    fn drop(&mut self) {
        self.mixer_count.fetch_sub(1, Ordering::SeqCst);
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

