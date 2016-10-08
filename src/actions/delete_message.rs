// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Message Delete Action ------------------------------------------------------
pub struct DeleteMessage {
    message: Message
}

impl DeleteMessage {
    pub fn new(message: Message) -> Box<DeleteMessage> {
        Box::new(DeleteMessage {
            message: message
        })
    }
}

impl Action for DeleteMessage {
    fn run(&self, _: &mut Bot, _: &BotConfig, queue: &EventQueue) -> ActionGroup {
        info!("{} Deleting...", self);
        queue.delete_message(self.message.id, self.message.channel_id);
        vec![]
    }
}

impl fmt::Display for DeleteMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [DeleteMessage] {}", self.message)
    }
}

