// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Sender};


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioReceiver;
use discord::model::UserId;


// Internal Dependencies ------------------------------------------------------
use super::queue::{Queue, QueueEntry, QueueHandle};


// Types ----------------------------------------------------------------------
pub type ListenerCount = Arc<AtomicUsize>;

pub struct EmptyListenerCount;

impl EmptyListenerCount {
    pub fn create() -> ListenerCount {
        Arc::new(AtomicUsize::new(0))
    }
}


// Voice Audio Listener -------------------------------------------------------
pub struct Listener {
    timer_thread: Option<thread::JoinHandle<()>>,
    peak_sender: Sender<Option<u32>>,
    queue_handle: Option<QueueHandle>,
    silence_threshold: u32
}

impl Listener {

    pub fn new(_: Queue, _: ListenerCount) -> Listener {

        let (peak_sender, peak_receive) = channel::<(Option<u32>)>();
        let (status_sender, status_receive) = channel::<(QueueEntry)>();
        let delay = Duration::from_millis(1000);
        let timer = thread::spawn(move || {

            let mut silent_for_seconds: usize = 0;
            loop {

                silent_for_seconds += 1;

                // Sample Peaks
                while let Ok(option) = peak_receive.try_recv() {
                    if let Some(_) = option {
                        silent_for_seconds = 0;

                    } else {
                        break;
                    }
                }

                // Status Commands
                while let Ok(status) = status_receive.try_recv() {
                    if let QueueEntry::Reset = status {
                        silent_for_seconds = 0;
                    }
                }

                if silent_for_seconds > 100 {
                    // TODO check if at least 3 people listening
                    silent_for_seconds = 0;
                }

                thread::sleep(delay);

            }

        });

        Listener {
            timer_thread: Some(timer),
            peak_sender: peak_sender,
            queue_handle: Some(status_sender),
            silence_threshold: 0
        }

    }

    pub fn take_handle(&mut self) -> Option<QueueHandle> {
        self.queue_handle.take()
    }

}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(timer_thread) = self.timer_thread.take() {
            self.peak_sender.send(None).ok();
            timer_thread.join().unwrap();
        }
    }
}


// Traits ---------------------------------------------------------------------
impl AudioReceiver for Listener {

    fn speaking_update(&mut self, _: u32, _: &UserId, _: bool) {}

    fn voice_packet(&mut self, _: u32, _: u16, _: u32, _: bool, data: &[i16]) {

        let peak = (*data.iter().max_by_key(|s| (**s as i32).abs()).unwrap_or(&0) as i32).abs() as u32;
        if peak > self.silence_threshold * 2 {
            self.peak_sender.send(Some(peak)).ok();
        }

        // TODO use bigger sliding window
        self.silence_threshold = cmp::max((self.silence_threshold + peak) / 2, 2000);

    }

}

