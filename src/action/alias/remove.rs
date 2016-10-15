// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message,
    name: String
}

impl ActionImpl {
    pub fn new(message: Message, name: String) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            name: name
        })
    }
}

impl Action for ActionImpl {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.remove_alias(&self.name);
            vec![MessageActions::Send::private(&self.message, format!(
                "Alias `{}` has been removed on {}.",
                self.name, server.name
            ))]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RemoveAlias] {}", self.name)
    }
}

