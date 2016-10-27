// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    server_id: ServerId,
    channel_id: ChannelId,
}

impl ActionImpl {
    pub fn new(
        server_id: ServerId,
        channel_id: ChannelId

    ) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            server_id: server_id,
            channel_id: channel_id
        })
    }
}

impl ActionHandler for ActionImpl {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.server_id) {
            info!("{} Starting audio recording...", self);
            // TODO get channel name and get public text channel?
            server.start_recording_voice(&self.channel_id, queue);
        }
        vec![]
    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [StartRecording] Server #{}",
            self.server_id
        )
    }
}

