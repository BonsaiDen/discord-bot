// Discord Dependencies -------------------------------------------------------
use discord::model::{
    ChannelId,
    UserId,
    User as DiscordUser,
    Member as DiscordMember,
    VoiceState as DiscordVoiceState
};


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::audio::MixerCommand;
use ::action::{ActionGroup, EffectActions};
use ::core::{EventQueue, Member};
use ::db::models::User as UserModel;
use super::{Server, ServerRecordingStatus, ServerVoiceStatus};


// Server Member Interface ----------------------------------------------------
enum VoiceStateResult {
    UpdateServerVoice,
    UpdateMemberVoice(bool),
    Ignore
}

impl Server {

    pub fn get_member(&self, member_id: &UserId) -> Option<&Member> {
        self.members.get(member_id)
    }

    pub fn has_member(&self, member_id: &UserId) -> bool {
        self.members.contains_key(member_id)
    }

    pub fn has_member_with_nickname(&self, nickname: &str) -> bool {
        self.members.values().any(|m| m.nickname == nickname)
    }

    pub fn add_member(
        &mut self,
        discord_member: DiscordMember,
        bot_config: &BotConfig
    ) {

        let mut member = Member::from_discord_member(
            discord_member,
            self.id,
            bot_config
        );

        let user = self.get_user_from_db(&member.nickname);
        member.is_admin = user.is_admin;
        member.is_uploader = user.is_uploader;
        member.is_banned = user.is_banned;

        info!("{} {} added", self, member);
        self.members.insert(member.id, member);

    }

    pub fn remove_member_from_user(&mut self, user: DiscordUser) {
        if let Some(member) = self.members.remove(&user.id) {
            info!("{} {} removed", self, member);
        }
    }

    pub fn update_member_voice_state(
        &mut self,
        voice_state: DiscordVoiceState,
        queue: &mut EventQueue,
        bot_config: &BotConfig

    ) -> ActionGroup {

        let actions = match self.apply_voice_state(&voice_state) {

            VoiceStateResult::UpdateServerVoice => {

                if self.voice_channel_id.is_some() {
                    if voice_state.channel_id.is_some() {
                        self.bot_left_voice();
                        self.bot_joined_voice(voice_state.channel_id.unwrap());

                    } else {
                        self.bot_left_voice();
                    }

                } else if voice_state.channel_id.is_some() {
                    self.bot_joined_voice(voice_state.channel_id.unwrap());
                }

                vec![]

            },

            VoiceStateResult::UpdateMemberVoice(true) => {
                self.greet_member(&voice_state, bot_config)
            },

            VoiceStateResult::UpdateMemberVoice(false) | VoiceStateResult::Ignore => {
                vec![]
            }

        };

        // Check if current server voice channel has become empty
        if let Some(channel_id) = self.voice_channel_id {

            let is_empty = {
                if let Some(channel) = self.channels.get(&channel_id) {
                    channel.is_empty_voice()

                } else {
                    false
                }
            };

            if is_empty {
                info!(
                    "{} Current voice channel has become vacant, leaving",
                    self
                );
                self.leave_voice(queue);
            }

        }

        actions

    }

    fn apply_voice_state(
        &mut self,
        voice_state: &DiscordVoiceState

    ) -> VoiceStateResult {

        let server = format!("{}", self);

        if let Some(member) = self.members.get_mut(&voice_state.user_id) {

            // Handle voice updates from active bot user
            if member.is_active_bot {
                VoiceStateResult::UpdateServerVoice

            // Ignore all other bots
            } else if member.is_bot {
                VoiceStateResult::Ignore

            } else {

                member.mute = voice_state.mute || voice_state.self_mute;
                member.deaf = voice_state.deaf || voice_state.self_deaf;

                let mut joined = false;
                if voice_state.channel_id != member.voice_channel_id {

                    // Leave old channel
                    if let Some(channel_id) = member.voice_channel_id {
                        if let Some(channel) = self.channels.get_mut(&channel_id) {
                            member.left_channel(&channel_id);
                            channel.remove_voice_member(&member.id);
                            info!("{} {} user {} left ", server, channel, member);
                        }
                    }

                    // Join new channel
                    if let Some(channel_id) = voice_state.channel_id {
                        if let Some(channel) = self.channels.get_mut(&channel_id) {
                            joined = true;
                            channel.add_voice_member(&member.id);
                            info!("{} {} user {} joined ", server, channel, member);
                        }
                    }

                    member.voice_channel_id = voice_state.channel_id;

                }

                info!("{} {} voice state updated", server, member);
                VoiceStateResult::UpdateMemberVoice(joined)

            }

        } else {
            VoiceStateResult::Ignore
        }

    }

    fn greet_member(
        &mut self,
        voice_state: &DiscordVoiceState,
        bot_config: &BotConfig

    ) -> ActionGroup {

        let now = clock_ticks::precise_time_ms();
        let channel_id = if now - self.startup_time < 1000 {
            info!("{} Ignored greeting for already connected member", self);
            None

        } else if let Some(member) = self.members.get_mut(&voice_state.user_id) {
            if member.should_be_greeted(bot_config) {
                Some(member.voice_channel_id.unwrap())

            } else {
                None
            }

        } else {
            None
        };

        if let Some(channel_id) = channel_id {
            if let Some(effects) = self.get_greeting(
                &voice_state.user_id,
                bot_config
            ) {
                vec![EffectActions::Play::new(
                    self.id,
                    channel_id,
                    effects,
                    false,
                    None
                )]

            } else {
                vec![]
            }

        } else {
            vec![]
        }

    }

    fn bot_joined_voice(&mut self, channel_id: ChannelId) {
        if self.voice_status == ServerVoiceStatus::Pending {

            info!("{} Joined voice channel", self);
            self.voice_status = ServerVoiceStatus::Joined;
            self.voice_channel_id = Some(channel_id);

            // Allow effects to be played once we actually joined the channel
            if let Some(queue) = self.mixer_commands.as_mut() {
                queue.send(MixerCommand::ClearDelay).ok();
            }

        }
    }

    fn bot_left_voice(&mut self) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} Left voice channel", self);
            self.voice_status = ServerVoiceStatus::Left;
            self.recording_status = ServerRecordingStatus::Stopped;
            self.pinned_channel_id = None;
            self.voice_channel_id = None;
        }
    }

    fn get_user_from_db(&self, nickname: &str) -> UserModel {

        use diesel::prelude::*;
        use ::db::schema::users::dsl::{server_id, nickname as user_nickname};
        use ::db::schema::users::table as userTable;

        userTable.filter(server_id.eq(&self.config.table_id))
                 .filter(user_nickname.eq(nickname))
                 .first::<UserModel>(&self.config.connection)
                 .ok()
                 .unwrap_or_else(|| {
                     UserModel {
                         id: -1,
                         server_id: self.config.table_id.clone(),
                         nickname: nickname.to_string(),
                         is_admin: false,
                         is_uploader: false,
                         is_banned: false
                     }
                 })
    }

}

