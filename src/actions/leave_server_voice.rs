// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct LeaveServerVoice {
    message: Message
}

impl LeaveServerVoice {
    pub fn new(message: Message) -> Box<LeaveServerVoice> {
        Box::new(LeaveServerVoice {
            message: message
        })
    }
}

impl Action for LeaveServerVoice {
    fn run(&self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.message.server_id) {
            info!("{} Leaving active voice channel...", self);
            server.leave_voice(queue);
        }
        vec![]
    }
}

impl fmt::Display for LeaveServerVoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [LeaveServerVoice] Server #{}",
            self.message.server_id
        )
    }
}


