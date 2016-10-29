// STD Dependencies -----------------------------------------------------------
use std::sync::mpsc;


// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;
use discord::model::permissions::{VOICE_CONNECT, VOICE_SPEAK};


// Internal Dependencies ------------------------------------------------------
use ::audio::{Mixer, MixerCommand, MixerEvent, Recorder};
use ::core::EventQueue;
use super::{Server, ServerRecordingStatus, ServerVoiceStatus};


// Server Voice Interface -----------------------------------------------------
impl Server {

    pub fn join_voice(
        &mut self,
        channel_id: &ChannelId,
        queue: &mut EventQueue

    ) -> bool {

        // Check voice channel permissions
        let permissions = self.get_bot_permissions(channel_id);
        if !permissions.contains(VOICE_CONNECT | VOICE_SPEAK) {
            info!("{} No permissions to join voice channel", self);
            return false;

        // Check if already pending
        } else if self.voice_status == ServerVoiceStatus::Pending {
            info!("{} Already joining a voice channel", self);
            return true;

        // Check if already in the target channel
        } else if let Some(current_channel_id) = self.voice_channel_id {
            if *channel_id == current_channel_id {
                info!("{} Already in target voice channel", self);
                return true;
            }
        }

        // Check if pinned to a specific voice channel
        if let Some(pinned_channel_id) = self.pinned_channel_id {
            // Allow re-join of current pinnted voice channel in case of disconnect
            if *channel_id != pinned_channel_id {
                info!("{} Pinned to the current voice channel", self);
                return false;
            }
        }

        // Check if we are currently recording audio
        if self.recording_status == ServerRecordingStatus::Recording {
            info!("{} Currently recording audio in a voice channel", self);
            return false;
        }

        if let Some(channel) = self.channels.get(channel_id) {
            info!("{} {} Joining voice", self, channel);
        }

        // Setup voice connection and mixer
        let (c_sender, c_receiver) = mpsc::channel::<MixerCommand>();
        let (e_sender, e_receiver) = mpsc::channel::<MixerEvent>();
        queue.connect_server_voice(self.id, *channel_id, move |conn| {
            conn.clear_receiver();
            conn.play(Box::new(Mixer::new(c_receiver, e_sender)));
        });

        self.mixer_commands = Some(c_sender);
        self.mixer_events = Some(e_receiver);
        self.voice_status = ServerVoiceStatus::Pending;

        true

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

    pub fn start_recording_voice(
        &mut self,
        channel_id: &ChannelId,
        queue: &mut EventQueue

    ) -> bool {

        self.join_voice(channel_id, queue);

        if self.voice_status != ServerVoiceStatus::Left {
            if self.recording_status == ServerRecordingStatus::Stopped {

                let path = self.config.recordings_path.clone();
                queue.with_server_voice(self.id, move |conn| {
                    conn.set_receiver(Box::new(Recorder::new(path, 1000)));
                });

                self.recording_status = ServerRecordingStatus::Recording;

                info!("{} Voice recording started", self);
                true

            } else {
                warn!("{} Voice recording could not be started: Recording is already active", self);
                false
            }

        } else {
            warn!("{} Voice recording could not be started: Failed to join target channel", self);
            false
        }
    }

    pub fn stop_recording_voice(&mut self, queue: &mut EventQueue) {
        if self.recording_status == ServerRecordingStatus::Recording {
            queue.with_server_voice(self.id, move |conn| {
                conn.clear_receiver();
            });
            self.recording_status = ServerRecordingStatus::Stopped;
            info!("{} Voice recording stopped", self);
        }
    }

    pub fn is_recording_voice(&self) -> bool {
        self.recording_status == ServerRecordingStatus::Recording
    }

}

