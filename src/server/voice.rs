// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;
use discord::model::permissions::{VOICE_CONNECT, VOICE_SPEAK};


// Internal Dependencies ------------------------------------------------------
use ::audio::Mixer;
use ::core::EventQueue;
use super::{Server, ServerVoiceStatus};


// Server Voice Interface -----------------------------------------------------
impl Server {

    pub fn join_voice(&mut self, channel_id: &ChannelId, queue: &mut EventQueue) {

        // Check voice channel permissions
        let permissions = self.get_bot_permissions(channel_id);
        if !permissions.contains(VOICE_CONNECT | VOICE_SPEAK) {
            info!("{} No permissions to join voice channel", self);
            return;
        }

        // Check if already pending
        if self.voice_status == ServerVoiceStatus::Pending {
            info!("{} Already joining a voice channel", self);
            return;

        // Check if already in the target channel
        } else if let Some(current_channel_id) = self.voice_channel_id {
            if *channel_id == current_channel_id {
                info!("{} Already in target voice channel", self);
                return;
            }
        }

        // Check if pinned to a specific voice channel
        if let Some(pinned_channel_id) = self.pinned_channel_id {
            // Allow re-join of current pinnted voice channel in case of disconnect
            if *channel_id != pinned_channel_id {
                info!("{} Pinned to the current voice channel", self);
                return;
            }
        }

        if let Some(channel) = self.channels.get(channel_id) {
            info!("{} {} Joining voice", self, channel);
        }

        let mixer_queue = self.mixer_queue.clone();
        queue.connect_server_voice(self.id, *channel_id, move |conn| {
            conn.play(Box::new(Mixer::new(mixer_queue)));
        });

        self.voice_status = ServerVoiceStatus::Pending;

    }

    pub fn update_voice(&mut self, _: &mut EventQueue) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} Voice endpoint updated", self);
        }
    }

    pub fn reconnect_voice(&mut self, queue: &mut EventQueue) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} Reconnecting voice...", self);
            let channel_id = self.voice_channel_id.take().unwrap();
            self.join_voice(&channel_id, queue);
        }
    }

    pub fn leave_voice(&mut self, queue: &mut EventQueue) {
        if let Some(channel_id) = self.voice_channel_id {

            if let Some(channel) = self.channels.get(&channel_id) {
                info!("{} {} Leaving voice", self, channel);
            }

            queue.disconnect_server_voice(self.id);

        }
    }

    pub fn pin_to_voice(&mut self) {
        if let Some(channel_id) = self.voice_channel_id {
            info!("{} Pinned to voice channel", self);
            self.pinned_channel_id = Some(channel_id);
        }
    }

    pub fn is_in_voice(&self) -> bool {
        self.voice_status == ServerVoiceStatus::Joined
    }

}

