// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::actions::SendMessage;
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct RemoveAlias {
    message: Message,
    name: String
}

impl RemoveAlias {
    pub fn new(message: Message, name: String) -> Box<RemoveAlias> {
        Box::new(RemoveAlias {
            message: message,
            name: name
        })
    }
}

impl Action for RemoveAlias {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.remove_alias(&self.name);
            // TODO message
            vec![]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for RemoveAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RemoveAlias] {}", self.name)
    }
}

