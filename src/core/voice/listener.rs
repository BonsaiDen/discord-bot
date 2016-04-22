// STD Dependencies -----------------------------------------------------------
use std::sync::mpsc::{channel, Sender};


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioReceiver;
use discord::model::UserId;


// Internal Dependencies ------------------------------------------------------
use super::queue::{Queue, QueueEntry, QueueHandle};


// Voice Audio Listener -------------------------------------------------------
pub struct Listener {
    queue_handle: Option<QueueHandle>,
}

impl Listener {

    pub fn new(_: Queue) -> Listener {

        let (status_sender, _) = channel::<(QueueEntry)>();

        Listener {
            queue_handle: Some(status_sender)
        }

    }

    pub fn take_handle(&mut self) -> Option<QueueHandle> {
        self.queue_handle.take()
    }

}


// Traits ---------------------------------------------------------------------
impl AudioReceiver for Listener {

    fn speaking_update(&mut self, _: u32, _: &UserId, _: bool) {}

    fn voice_packet(&mut self, _: u32, _: u16, _: u32, _: bool, _: &[i16]) {}

}

