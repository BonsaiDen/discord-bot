// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Server Configuration Reload Action -----------------------------------------
pub struct ReloadServerConfiguration {
    message: Message
}

impl ReloadServerConfiguration {
    pub fn new(message: Message) -> Box<ReloadServerConfiguration> {
        Box::new(ReloadServerConfiguration {
            message: message
        })
    }
}

impl Action for ReloadServerConfiguration {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.reload();
        }
        vec![]
    }
}

impl fmt::Display for ReloadServerConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [ReloadServerConfiguration] Server #{}",
            self.message.server_id
        )
    }
}


