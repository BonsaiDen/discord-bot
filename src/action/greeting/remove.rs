// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message,
    nickname: String
}

impl ActionImpl {
    pub fn new(message: Message, nickname: String) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            nickname: nickname
        })
    }
}

impl Action for ActionImpl {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.remove_greeting(&self.nickname);
            MessageActions::Send::private(&self.message, format!(
                "Greeting for `{}` has been removed on {}.",
                self.nickname, server.name
            ))

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RemoveGreeting] {}", self.nickname)
    }
}

