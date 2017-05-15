// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::sync::mpsc;
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
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::audio::{MixerCommand, MixerEvent};
use ::bot::BotConfig;
use ::core::{Channel, EventQueue, Member};
use ::effect::EffectRegistry;


// Modules --------------------------------------------------------------------
mod actions;
mod channel;
mod config;
mod effect;
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

#[derive(Debug, PartialEq)]
pub enum ServerRecordingStatus {
    Recording,
    Stopped
}


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    pub id: ServerId,
    pub name: String,

    region: String,
    config: ServerConfig,
    startup_time: u64,

    effects: EffectRegistry,
    aliases: HashMap<String, Vec<String>>,
    voice_channel_id: Option<ChannelId>,
    pinned_channel_id: Option<ChannelId>,
    voice_status: ServerVoiceStatus,
    recording_status: ServerRecordingStatus,

    mixer_commands: Option<mpsc::Sender<MixerCommand>>,
    mixer_events: Option<mpsc::Receiver<MixerEvent>>,

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
                    aliases: HashMap::new(),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    pinned_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    recording_status: ServerRecordingStatus::Stopped,
                    mixer_commands: None,
                    mixer_events: None,
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
                    aliases: HashMap::new(),
                    effects: EffectRegistry::new(),
                    voice_channel_id: None,
                    pinned_channel_id: None,
                    voice_status: ServerVoiceStatus::Left,
                    recording_status: ServerRecordingStatus::Stopped,
                    mixer_commands: None,
                    mixer_events: None,
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

                // TODO why was sync_members needed in the first place?

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
        self.update_aliases();
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

    pub fn events(&self) -> Vec<MixerEvent> {
        if let Some(ref mixer_events) = self.mixer_events {
            let mut events = Vec::new();
            while let Ok(event) = mixer_events.try_recv() {
                events.push(event)
            }
            events

        } else {
            vec![]
        }
    }

}


// Internal Interface ---------------------------------------------------------
impl Server {

    fn add_discord_channel(&mut self, channel: DiscordChannel) {
        self.add_channel(Channel::from_discord_channel(channel));
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

