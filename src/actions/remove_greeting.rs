// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::actions::SendMessage;
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct RemoveGreeting {
    message: Message,
    nickname: String
}

impl RemoveGreeting {
    pub fn new(message: Message, nickname: String) -> Box<RemoveGreeting> {
        Box::new(RemoveGreeting {
            message: message,
            nickname: nickname
        })
    }
}

impl Action for RemoveGreeting {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.remove_greeting(&self.nickname);
            // TODO message
            vec![]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for RemoveGreeting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RemoveGreeting] {}", self.nickname)
    }
}

