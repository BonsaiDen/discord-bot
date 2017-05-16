// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message,
    name: String
}

impl Action {
    pub fn new(message: Message, name: String) -> Box<Action> {
        Box::new(Action {
            message: message,
            name: name
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.add_streamer(&self.name);
            MessageActions::Send::private(&self.message, format!(
                "Twitch streamer `{}` is now being watched on on {}.",
                self.name, server.name
            ))

        } else {
            vec![]
        }

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [AddStreamer] {}",
            self.name
        )
    }
}

