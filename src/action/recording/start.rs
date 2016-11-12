// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    server_id: ServerId,
    voice_channel_id: ChannelId,
}

impl Action {
    pub fn new(
        server_id: ServerId,
        voice_channel_id: ChannelId

    ) -> Box<Action> {
        Box::new(Action {
            server_id: server_id,
            voice_channel_id: voice_channel_id
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {

        let mut actions: Vec<Box<ActionHandler>> = Vec::new();

        if let Some(server) = bot.get_server(&self.server_id) {

            info!("{} Starting audio recording...", self);

            if let Some(channel_name) = server.channel_name(&self.voice_channel_id) {

                // Notify all users in the current voice channel
                for member in server.channel_voice_members(&self.voice_channel_id) {
                    actions.push(MessageActions::Send::user_private(
                        member.id,
                        format!(
                            "Note: Audio recording has been **started** for your current voice channel {}.",
                            channel_name
                        )
                    ))
                }

                server.start_recording_voice(&self.voice_channel_id, queue);

            }

        }

        actions

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [StartRecording] Server #{}",
            self.server_id
        )
    }
}

