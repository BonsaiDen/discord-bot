// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    ChannelId, UserId, ServerId,
    User as DiscordUser,
    Member as DiscordMember,
    Server as DiscordServer,
    Channel as DiscordChannel,
    VoiceState as DiscordVoiceState,
    PossibleServer,
    LiveServer,
    Permissions
};
use discord::model::permissions::{VOICE_CONNECT, VOICE_SPEAK};


// External Dependencies ------------------------------------------------------
use toml;
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::audio::{Mixer, MixerCommand, MixerQueue, EmptyMixerQueue};
use ::bot::BotConfig;
use ::core::event::EventQueue;
use ::core::member::Member;
use ::core::channel::Channel;
use ::effect::{Effect, EffectRegistry};
use ::action::{ActionGroup, EffectActions};


// Modules --------------------------------------------------------------------
mod config;


// Re-Exports -----------------------------------------------------------------
pub use self::config::ServerConfig;


// Server Voice Abstraction ---------------------------------------------------
#[derive(Debug, PartialEq)]
enum ServerVoiceStatus {
    Pending,
    Joined,
    Left
}


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    pub id: ServerId,
    pub name: String,

    region: String,
    config: ServerConfig,
    startup_time: u64,

    effects: EffectRegistry,
    voice_channel_id: Option<ChannelId>,
    // pinned_channel_id: Option<ChannelId>, TODO allow for channel pinning
    voice_status: ServerVoiceStatus,
    mixer_queue: MixerQueue,

    channels: HashMap<ChannelId, Channel>,
    members: HashMap<UserId, Member>

}


// Public Interface -----------------------------------------------------------
impl Server {

    pub fn id_from_possible_server(server: &PossibleServer<LiveServer>) -> ServerId {
        match *server {
            PossibleServer::Offline(server_id) => server_id,
            PossibleServer::Online(ref live_server) => live_server.id
        }
    }

    pub fn from_possible_server(
        server: PossibleServer<LiveServer>,
        bot_config: &BotConfig,
        queue: &mut EventQueue

    ) -> Server {
        let server = match server {

            PossibleServer::Offline(server_id) => {
                Server {
                    id: server_id,
                    name: "".to_string(),
                    region: "".to_string(),
                    startup_time: clock_ticks::precise_time_ms(),
                    config: ServerConfig::new(&server_id, bot_config),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    mixer_queue: EmptyMixerQueue::create(),
                    channels: HashMap::new(),
                    members: HashMap::new()
                }
            },

            PossibleServer::Online(live_server) => {

                let mut server = Server {
                    id: live_server.id,
                    name: live_server.name,
                    region: live_server.region,
                    startup_time: clock_ticks::precise_time_ms(),
                    config: ServerConfig::new(&live_server.id, bot_config),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    mixer_queue: EmptyMixerQueue::create(),
                    channels: HashMap::new(),
                    members: HashMap::new()
                };

                server.reload();

                for member in live_server.members {
                    server.add_member(member, bot_config);
                }

                for channel in live_server.channels {
                    server.add_discord_channel(DiscordChannel::Public(channel));
                }

                for voice_state in live_server.voice_states {
                    server.update_member_voice_state(
                        voice_state,
                        queue,
                        bot_config
                    );
                }

                server

            }

        };

        info!("{} Initiated.", server);

        server

    }

    pub fn update(&mut self, server: DiscordServer) {
        self.name = server.name;
        self.region = server.region;
        info!("{} Updated", self);
    }

    pub fn reload(&mut self) {

        match self.load_config() {
            Ok(_) => info!("{} Configuration loaded", self),
            Err(reason) => warn!("{} Failed to load configuration: {}", self, reason)
        }

        self.effects.reload(&self.config);

    }

}


// Voice Interface ------------------------------------------------------------
impl Server {

    fn join_voice(&mut self, channel_id: &ChannelId, queue: &mut EventQueue) {

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

    fn joined_voice(&mut self, channel_id: ChannelId) {
        if self.voice_status == ServerVoiceStatus::Pending {
            info!("{} Joined voice channel", self);
            self.voice_status = ServerVoiceStatus::Joined;
            self.voice_channel_id = Some(channel_id);
        }
    }

    fn left_voice(&mut self) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} Left voice channel", self);
            self.voice_status = ServerVoiceStatus::Left;
            self.voice_channel_id = None;
        }
    }

}


// Aliases Interface ----------------------------------------------------------
impl Server {

    pub fn has_alias(&self, alias_name: &str) -> bool {
        self.config.aliases.contains_key(alias_name)
    }

    #[allow(ptr_arg)]
    pub fn add_alias(&mut self, alias_name: &str, effect_names: &Vec<String>) {
        self.config.aliases.insert(alias_name.to_string(), effect_names.clone());
        self.store_config().expect("add_alias failed to store config.");
    }

    pub fn remove_alias(&mut self, alias_name: &str) {
        self.config.aliases.remove(alias_name);
        self.store_config().expect("remove_alias failed to store config.");
    }

    pub fn list_aliases(&self) -> Vec<(&String, &Vec<String>)> {
        self.config.aliases.iter().map(|(name, effects)| {
            (name, effects)

        }).collect()
    }

}


// Greetings Interface --------------------------------------------------------
impl Server {

    pub fn get_greeting(
        &self,
        member_id: &UserId,
        bot_config: &BotConfig

    ) -> Option<Vec<&Effect>> {
        if let Some(member) = self.members.get(member_id) {
            if let Some(effect_name) = self.config.greetings.get(&member.nickname) {
                Some(self.map_effects(
                    &[effect_name.to_string()],
                    false,
                    bot_config
                ))

            } else if let Some(effect_name) = self.config.greetings.get("effect") {
                Some(self.map_effects(
                    &[effect_name.to_string()],
                    false,
                    bot_config
                ))

            } else {
                None
            }

        } else {
            None
        }
    }

    pub fn has_greeting(&self, nickname: &str) -> bool {
        self.config.greetings.contains_key(nickname)
    }

    pub fn add_greeting(&mut self, nickname: &str, effect_name: &str) {
        self.config.greetings.insert(nickname.to_string(), effect_name.to_string());
        self.store_config().expect("add_greeting failed to store config.");
    }


    pub fn remove_greeting(&mut self, nickname: &str) {
        self.config.greetings.remove(nickname);
        self.store_config().expect("remove_greeting failed to store config.");
    }

    pub fn list_greetings(&self) -> Vec<(&String, &String)> {
        self.config.greetings.iter().map(|(nickname, effect)| {
            (nickname, effect)

        }).collect()
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
                vec![EffectActions::Play::new(self.id, channel_id, effects, false)]

            } else {
                vec![]
            }

        } else {
            vec![]
        }

    }

}


// Effects Interface ----------------------------------------------------------
impl Server {

    pub fn play_effects(
        &mut self,
        channel_id: &ChannelId,
        effects: &[Effect],
        queued: bool,
        queue: &mut EventQueue
    ) {

        let has_channel = if let Some(channel) = self.channels.get(channel_id) {
            info!("{} {} playing {} effect(s)...", self, channel, effects.len());
            true

        } else {
            false
        };

        if has_channel {

            self.join_voice(channel_id, queue);

            if let Ok(mut queue) = self.mixer_queue.lock() {
                if queued {
                    queue.push_back(MixerCommand::QueueEffects(effects.to_vec()));

                } else {
                    queue.push_back(MixerCommand::PlayEffects(effects.to_vec()));
                }
            }

        }

    }

    pub fn silence_active_effects(&mut self) {
        if let Ok(mut queue) = self.mixer_queue.lock() {
            queue.push_back(MixerCommand::ClearQueue);
        }
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.effects.has_effect(effect_name)
    }

    pub fn get_effect(&self, effect_name: &str) -> Option<&Effect> {
        self.effects.get_effect(effect_name)
    }

    pub fn has_matching_effects(&self, effect_name: &str, bot_config: &BotConfig) -> bool {
        !self.map_effects(
            &[effect_name.to_string()],
            true,
            bot_config

        ).is_empty()
    }

    pub fn map_effects(
        &self,
        patterns: &[String],
        match_all: bool,
        bot_config: &BotConfig

    ) -> Vec<&Effect> {
        self.effects.map_patterns(
            patterns, Some(&self.config.aliases), match_all, bot_config
        )
    }

    pub fn map_similiar_effects(&self, patterns: &[String]) -> Vec<&str> {
        self.effects.map_similiar(
            patterns,
            &self.config.aliases
        )
    }

    pub fn rename_effect(&mut self, effect: &Effect, effect_name: &str) -> Result<(), String> {
        self.effects.rename_effect(&self.config, effect, effect_name)
    }

    pub fn delete_effect(&mut self, effect: &Effect) -> Result<(), String> {
        self.effects.delete_effect(&self.config, effect)
    }

    pub fn download_effect(
        &mut self,
        effect_name: &str,
        upload_url: &str,
        uploader: &str

    ) -> Result<(), String> {
        self.effects.download_effect(
            &self.config,
            effect_name,
            upload_url,
            uploader
        )
    }

}


// Member Interface -----------------------------------------------------------
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
        member.is_admin = self.config.admins.contains(&member.nickname);
        member.is_uploader = self.config.uploaders.contains(&member.nickname);

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
                        self.left_voice();
                        self.joined_voice(voice_state.channel_id.unwrap());

                    } else {
                        self.left_voice();
                    }

                } else if voice_state.channel_id.is_some() {
                    self.joined_voice(voice_state.channel_id.unwrap());
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

}


// Ban Interface --------------------------------------------------------------
impl Server {

    pub fn list_bans(&self) -> Vec<String> {
        self.config.banned.iter().map(|n| n.to_string()).collect()
    }

    #[allow(ptr_arg)]
    pub fn add_ban(&mut self, nickname: &String) -> bool {
        if !self.config.banned.contains(nickname) {
            self.config.banned.push(nickname.to_string());
            self.store_config().expect("add_ban failed to store config.");
            true

        } else {
            false
        }
    }

    #[allow(ptr_arg)]
    pub fn remove_ban(&mut self, nickname: &String) -> bool {
        if self.config.banned.contains(nickname) {
            self.config.banned.retain(|n| n != nickname);
            self.store_config().expect("remove_ban failed to store config.");
            true

        } else {
            false
        }
    }

}


// Bot Interface --------------------------------------------------------------
impl Server {

    pub fn get_bot(&self) -> Option<&Member> {
        self.members.values().find(|m| m.is_active_bot)
    }

    pub fn get_bot_permissions(&self, channel_id: &ChannelId) -> Permissions {

        if let Some(channel) = self.channels.get(channel_id) {
            if let Some(member) = self.get_bot() {
                channel.get_member_permissions(member)

            } else {
                Permissions::empty()
            }

        } else {
            Permissions::empty()
        }

    }

}


// Channel Interface ----------------------------------------------------------
impl Server {

    pub fn has_channel(&self, channel_id: &ChannelId) -> bool {
        self.channels.contains_key(channel_id)
    }

    fn add_discord_channel(&mut self, channel: DiscordChannel) {
        self.add_channel(Channel::from_discord_channel(channel));
    }

    pub fn add_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        self.channels.insert(channel_id, channel);
        info!("{} {} added", self, self.channels.get(&channel_id).unwrap());
    }

    pub fn update_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        if self.channels.contains_key(&channel_id) {
            if let Some(server_channel) = self.channels.get_mut(&channel.id) {
                server_channel.update(channel);
            }
            info!("[{}] {} updated", self, self.channels.get(&channel_id).unwrap());
        }
    }

    pub fn remove_channel(&mut self, channel: Channel) {
        self.channels.remove(&channel.id);
        info!("{} {} removed", self, channel);
    }

}

// Config Interface -----------------------------------------------------------
impl Server {

    fn load_config(&mut self) -> Result<(), String> {
        self.config.ensure_directory()
            .and_then(|_| {
                File::open(self.config.config_path.clone())
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)
                            .map_err(|err| err.to_string())
                            .map(|_| buffer)

                    })
            })
            .and_then(|buffer| {
                toml::Parser::new(&buffer)
                    .parse()
                    .map_or_else(|| {
                        Err("Failed to parse configuration toml.".to_string())

                    }, |value| {
                        self.config.decode_from_toml(value);
                        self.sync_config();
                        Ok(())
                    })
            })
    }

    fn store_config(&mut self) -> Result<(), String> {

        self.sync_config();

        self.config.ensure_directory()
            .and_then(|_| {
                File::create(self.config.config_path.clone())
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        write!(file, "{}", self.config.encode_to_toml())
                            .map_err(|err| err.to_string())
                    })
            })

    }

    fn sync_config(&mut self) {
        for mut member in self.members.values_mut() {
            member.is_admin = self.config.admins.contains(&member.nickname);
            member.is_uploader = self.config.uploaders.contains(&member.nickname);
            member.is_banned = self.config.banned.contains(&member.nickname);
        }
    }

}


// Traits ---------------------------------------------------------------------
impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Server {} {} / {}]",
            self.name, self.channels.len(), self.members.len()
        )
    }
}

