// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message
}

impl ActionImpl {
    pub fn new(message: Message) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message
        })
    }
}

impl Action for ActionImpl {
    fn run(&self, _: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        info!("{} Deleting...", self);
        queue.delete_message(self.message.id, self.message.channel_id);
        vec![]
    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [DeleteMessage] {}", self.message)
    }
}

