// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    ChannelId, UserId, ServerId,
    Server as DiscordServer,
    Channel as DiscordChannel,
    PossibleServer,
    LiveServer,
    Permissions
};


// External Dependencies ------------------------------------------------------
use toml;
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::audio::{MixerQueue, EmptyMixerQueue};
use ::bot::BotConfig;
use ::core::{Channel, EventQueue, Member};
use ::effect::EffectRegistry;


// Modules --------------------------------------------------------------------
mod alias;
mod ban;
mod channel;
mod config;
mod effect;
mod greeting;
mod member;
mod voice;


// Re-Exports -----------------------------------------------------------------
pub use self::config::ServerConfig;


// Server Voice Abstraction ---------------------------------------------------
#[derive(Debug, PartialEq)]
pub enum ServerVoiceStatus {
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
    pinned_channel_id: Option<ChannelId>,
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
                    pinned_channel_id: None,
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
                    pinned_channel_id: None,
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


// Internal Interface ---------------------------------------------------------
impl Server {

    fn add_discord_channel(&mut self, channel: DiscordChannel) {
        self.add_channel(Channel::from_discord_channel(channel));
    }

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

