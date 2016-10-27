// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup};


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

impl ActionHandler for ActionImpl {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.silence_active_effects()
        }
        vec![]
    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [SilenceActiveEffects] Server #{}",
            self.message.server_id
        )
    }
}

