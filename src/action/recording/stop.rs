// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct Action {
    server_id: ServerId
}

impl Action {
    pub fn new(
        server_id: ServerId

    ) -> Box<Action> {
        Box::new(Action {
            server_id: server_id
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.server_id) {
            info!("{} Stopping audio recording...", self);
            server.stop_recording_voice(queue);
        }
        vec![]
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [StopRecording] Server #{}",
            self.server_id
        )
    }
}
