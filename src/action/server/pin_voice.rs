// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message
}

impl Action {
    pub fn new(message: Message) -> Box<Action> {
        Box::new(Action {
            message: message
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.message.server_id) {
            info!("{} Pinning to current voice channel...", self);
            server.pin_to_voice();
        }
        vec![]
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [PinServerVoice] Server #{}",
            self.message.server_id
        )
    }
}

