// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::collections::{HashMap, BTreeMap};


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    ChannelId, UserId, ServerId,
    Role,
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


// Internal Dependencies ------------------------------------------------------
use ::audio::{Mixer, MixerCommand, MixerQueue, EmptyMixerQueue};
use ::bot::BotConfig;
use ::core::event::EventQueue;
use ::core::member::Member;
use ::core::channel::Channel;
use ::effects::{Effect, EffectRegistry};


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    pub id: ServerId,
    pub name: String,

    region: String,
    config: ServerConfig,

    effects: EffectRegistry,
    voice_channel_id: Option<ChannelId>,
    voice_status: ServerVoiceStatus,
    mixer_queue: MixerQueue,

    channels: HashMap<ChannelId, Channel>,
    members: HashMap<UserId, Member>,
    roles: Vec<Role>

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
                    config: ServerConfig::new(&server_id, bot_config),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    mixer_queue: EmptyMixerQueue::create(),
                    channels: HashMap::new(),
                    members: HashMap::new(),
                    roles: Vec::new()
                }
            },

            PossibleServer::Online(live_server) => {

                let mut server = Server {
                    id: live_server.id,
                    name: live_server.name,
                    region: live_server.region,
                    config: ServerConfig::new(&live_server.id, bot_config),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    mixer_queue: EmptyMixerQueue::create(),
                    channels: HashMap::new(),
                    members: HashMap::new(),
                    roles: live_server.roles
                };

                server.reload();

                for member in live_server.members.into_iter() {
                    server.add_member(member, bot_config);
                }

                for channel in live_server.channels.into_iter() {
                    server.add_discord_channel(DiscordChannel::Public(channel));
                }

                for voice_state in live_server.voice_states.into_iter() {
                    server.update_member_voice_state(voice_state, queue);
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
        self.roles = server.roles;
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

    fn join_voice(
        &mut self,
        channel_id: &ChannelId,
        queue: &mut EventQueue
    ) {

        // Check voice channel permissions
        let permissions = self.get_bot_permissions(channel_id);
        if !permissions.contains(VOICE_CONNECT | VOICE_SPEAK) {
            info!("{} no permissions to join voice channel", self);
            return;
        }

        // Check if already pending
        if self.voice_status == ServerVoiceStatus::Pending {
            info!("{} already joining a voice channel", self);
            return;

        // Check if already in the target channel
        } else if let Some(current_channel_id) = self.voice_channel_id {
            if *channel_id == current_channel_id {
                info!("{} already in target voice channel", self);
                return;
            }
        }

        if let Some(channel) = self.channels.get(channel_id) {
            info!("{} {} joining voice", self, channel);
        }

        let mixer_queue = self.mixer_queue.clone();
        queue.connect_server_voice(self.id, *channel_id, move |conn| {
            conn.play(Box::new(Mixer::new(mixer_queue)));
        });

        self.voice_status = ServerVoiceStatus::Pending;

    }

    pub fn update_voice(&mut self, _: &mut EventQueue) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} ============ voice endpoint updated", self);
            // TODO this doesn't work since we're marked as joined BEFORE
            // this event is raised we need to have some delay check here
            //self.leave_voice(queue);
            // TODO re-join voice channel after region switch or re-connect
        }
    }

    pub fn leave_voice(&mut self, queue: &mut EventQueue) {
        if let Some(channel_id) = self.voice_channel_id {

            if let Some(channel) = self.channels.get(&channel_id) {
                info!("{} {} leaving voice", self, channel);
            }

            queue.disconnect_server_voice(self.id);

        }
    }

    fn joined_voice(&mut self, channel_id: ChannelId) {
        if self.voice_status == ServerVoiceStatus::Pending {
            info!("{} ============ Joined voice channel", self);
            self.voice_status = ServerVoiceStatus::Joined;
            self.voice_channel_id = Some(channel_id);
        }
    }

    fn left_voice(&mut self) {
        if self.voice_status == ServerVoiceStatus::Joined {
            info!("{} ============ Left voice channel", self);
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

    //pub fn add_alias(&mut self, alias_name: String, effect_names: Vec<String>) {
    //    self.config.aliases.insert(alias_name, effect_names);
    //    self.store_config();
    //}

    //pub fn remove_alias(&mut self, alias_name: &String) {
    //    self.config.aliases.remove(alias_name);
    //    self.store_config();
    //}

    pub fn list_aliases(&self) -> Vec<(&String, &Vec<String>)> {
        self.config.aliases.iter().map(|(name, effects)| {
            (name, effects)

        }).collect()
    }

}


// Greetings Interface --------------------------------------------------------
impl Server {


    pub fn has_greeting(&self, nickname: &str) -> bool {
        self.config.greetings.contains_key(nickname)
    }

    //pub fn add_alias(&mut self, nickname: String, effect_names: Vec<String>) {
    //    self.config.greetings.insert(nickname, effect_names);
    //    self.store_config();
    //}

    //pub fn remove_greeting(&mut self, nickname: &String) {
    //    self.config.greeting.remove(nickname);
    //    self.store_config();
    //}

    pub fn list_greetings(&self) -> Vec<(&String, &String)> {
        self.config.greetings.iter().map(|(nickname, effect)| {
            (nickname, effect)

        }).collect()
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

    pub fn get_effect(&self, effect_name: &str) -> Option<Effect> {
        self.effects.get_effect(effect_name)
    }

    pub fn map_effects(
        &self,
        patterns: &[String],
        match_all: bool,
        config: &BotConfig

    ) -> Vec<Effect> {
        self.effects.map_patterns(
            patterns, Some(&self.config.aliases), match_all, config
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
impl Server {

    pub fn get_member(&self, member_id: &UserId) -> Option<&Member> {
        self.members.get(member_id)
    }

    pub fn has_member(&self, member_id: &UserId) -> bool {
        self.members.contains_key(member_id)
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
        queue: &mut EventQueue
    ) {

        let server = format!("{}", self);

        let bot_voice_update = if let Some(member) = self.members.get_mut(&voice_state.user_id) {

            // Handle active bot user
            if member.is_active_bot {
                true

            // Ignore all other bots
            } else if member.is_bot {
                false

            } else {

                member.mute = voice_state.mute || voice_state.self_mute;
                member.deaf = voice_state.deaf || voice_state.self_deaf;

                if voice_state.channel_id != member.voice_channel_id {

                    // Leave old channel
                    if let Some(channel_id) = member.voice_channel_id {
                        if let Some(channel) = self.channels.get_mut(&channel_id) {
                            channel.remove_voice_member(&member.id);
                            info!("{} {} user {} left ", server, channel, member);
                        }
                    }

                    // Join new channel
                    if let Some(channel_id) = voice_state.channel_id {
                        if let Some(channel) = self.channels.get_mut(&channel_id) {
                            channel.add_voice_member(&member.id);
                            info!("{} {} user {} joined ", server, channel, member);
                        }
                    }

                    // TODO run greeting logic (return a Action for playing the effect?)
                    member.voice_channel_id = voice_state.channel_id;

                }

                info!("{} {} voice state updated", server, member);
                false

            }

        } else {
            false
        };

        // Handle Bot Voice Channel Updates
        if bot_voice_update {

            if self.voice_channel_id.is_some() {
                if voice_state.channel_id.is_some() {
                    self.left_voice();
                    self.joined_voice(voice_state.channel_id.unwrap());

                } else {
                    self.left_voice();
                }

            } else if voice_state.channel_id.is_some() {
                self.joined_voice(voice_state.channel_id.unwrap());
            }

        // Check if current server voice channel has become empty
        } else if let Some(channel_id) = self.voice_channel_id {

            let is_empty = {
                if let Some(channel) = self.channels.get(&channel_id) {
                    channel.is_empty_voice()

                } else {
                    false
                }
            };

            if is_empty {
                info!("{} Current voice channel has become vacant, leaving", server);
                self.leave_voice(queue);
            }

        }

    }

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


// Server Voice Abstraction ---------------------------------------------------
#[derive(Debug, PartialEq)]
enum ServerVoiceStatus {
    Pending,
    Joined,
    Left
}


// Server Configuration Abstraction -------------------------------------------
#[derive(Debug)]
pub struct ServerConfig {
    pub config_path: PathBuf,
    pub effects_path: PathBuf,
    aliases: HashMap<String, Vec<String>>,
    greetings: HashMap<String, String>,
    uploaders: Vec<String>,
    admins: Vec<String>
}

impl ServerConfig {

    fn new(server_id: &ServerId, bot_config: &BotConfig) -> Self {

        let mut config_path = bot_config.config_path.clone();
        config_path.push(server_id.0.to_string());
        config_path.push("config");
        config_path.set_extension("toml");

        let mut effects_path = bot_config.config_path.clone();
        effects_path.push(server_id.0.to_string());
        effects_path.push("effects");

        ServerConfig {
            config_path: config_path,
            effects_path: effects_path,
            aliases: HashMap::new(),
            greetings: HashMap::new(),
            admins: Vec::new(),
            uploaders: Vec::new()
        }

    }

    fn ensure_directory(&self) -> Result<(), String> {
        fs::create_dir_all(
            self.config_path.clone().parent().unwrap()

        ).map_err(|err| err.to_string())
    }

    fn encode_to_toml(&self) -> toml::Value {

        let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

        let list = toml::Value::Array(self.admins.iter().map(|nickname| {
            toml::Value::String(nickname.to_string())

        }).collect());

        toml.insert("admins".to_string(), list);

        let list = toml::Value::Array(self.uploaders.iter().map(|nickname| {
            toml::Value::String(nickname.to_string())

        }).collect());

        toml.insert("uploaders".to_string(), list);

        let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
        for (nickname, effect) in &self.greetings {
            table.insert(
                nickname.clone(),
                toml::Value::String(effect.clone())
            );
        }
        toml.insert("greetings".to_string(), toml::Value::Table(table));

        let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
        for (alias, effects) in &self.aliases {
            table.insert(
                alias.clone(),
                toml::Value::Array(effects.iter().map(|e| {
                    toml::Value::String(e.to_string())

                }).collect())
            );
        }
        toml.insert("aliases".to_string(), toml::Value::Table(table));

        toml::Value::Table(toml)

    }

    fn decode_from_toml(&mut self, value: BTreeMap<String, toml::Value>) {

        self.aliases.clear();
        self.greetings.clear();
        self.admins.clear();
        self.uploaders.clear();

        if let Some(&toml::Value::Table(ref table)) = value.get("aliases") {
            for (alias, names) in table {
                if let toml::Value::Array(ref names) = *names {
                    let mut effects: Vec<String> = Vec::new();
                    for name in names {
                        if let toml::Value::String(ref name) = *name {
                            effects.push(name.clone());
                        }
                    }
                    self.aliases.insert(alias.clone(), effects);
                }
            }
        }

        if let Some(&toml::Value::Table(ref table)) = value.get("greetings") {
            for (nickname, effect) in table {
                if let toml::Value::String(ref effect) = *effect {
                    self.greetings.insert(
                        nickname.clone(),
                        effect.clone()
                    );
                }
            }
        }

        if let Some(&toml::Value::Array(ref nicknames)) = value.get("admins") {
            for nickname in nicknames {
                if let toml::Value::String(ref nickname) = *nickname {
                    self.admins.push(nickname.clone());
                }
            }
        }

        if let Some(&toml::Value::Array(ref nicknames)) = value.get("uploaders") {
            for nickname in nicknames {
                if let toml::Value::String(ref nickname) = *nickname {
                    self.uploaders.push(nickname.clone());
                }
            }
        }

    }

}

