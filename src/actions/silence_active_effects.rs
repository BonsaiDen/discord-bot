// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Server Effect Silencing Action ---------------------------------------------
pub struct SilenceActiveEffects {
    message: Message
}

impl SilenceActiveEffects {
    pub fn new(message: Message) -> Box<SilenceActiveEffects> {
        Box::new(SilenceActiveEffects {
            message: message
        })
    }
}

impl Action for SilenceActiveEffects {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.message.server_id) {
            info!("{} Silencing effect mixer...", self);
            server.silence_active_effects()
        }
        vec![]
    }
}

impl fmt::Display for SilenceActiveEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [SilenceActiveEffects] Server #{}",
            self.message.server_id
        )
    }
}

