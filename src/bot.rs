// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    MessageId, ChannelId, ServerId, UserId,
    Attachment,
    User as DiscordUser,
    Event as DiscordEvent
};


// Internal Dependencies ------------------------------------------------------
use upload::Upload;
use command::Command;
use actions::ActionGroup;

use core::server::Server;
use core::member::Member;
use core::channel::Channel;
use core::event::{EventQueue, Event};
use core::message::{Message, MessageKind, MessageOrigin};


// Bot Configuration ----------------------------------------------------------
pub struct BotConfig {
    pub bot_nickname: String,
    pub server_whitelist: Vec<ServerId>,
    pub config_path: PathBuf,
    pub effect_playback_separation_ms: u64,
    pub flac_max_size: u64,
    pub flac_sample_rate: u32,
    pub flac_bits_per_sample: u8
}


// Discord Bot Implementation -------------------------------------------------
pub struct Bot {
    servers: HashMap<ServerId, Server>
}


// Public Interface -----------------------------------------------------------
impl Bot {

    pub fn create(token: String, config: BotConfig) {

        let bot = Bot {
            servers: HashMap::new()
        };

        bot.run(token, config);

    }

    pub fn get_server(&mut self, server_id: &ServerId) -> Option<&mut Server> {
        self.servers.get_mut(server_id)
    }

}


// Internal Interface ---------------------------------------------------------
impl Bot {

    fn run(mut self, token: String, config: BotConfig) {

        let mut queue = EventQueue::new(token);

        'main: loop {

            for event in queue.events() {

                // Handle Events
                let mut actions = match event {
                    Event::Disconnected => {
                        break 'main;
                    },
                    Event::Received(event) => self.discord_event(event, &config, &mut queue),
                    _ => self.event(event)
                };

                // Run resulting Actions
                while !actions.is_empty() {
                    info!("[Bot] Running {} actions...", actions.len());

                    let mut next_actions = Vec::new();
                    for action in actions.drain(0..) {
                        next_actions.extend(action.run(&mut self, &config, &mut queue));
                    }

                    actions.extend(next_actions);

                }

            }

            thread::sleep(Duration::from_millis(50));

        }

        queue.shutdown();

    }

}


// Event Handling -------------------------------------------------------------
impl Bot {

    fn event(&mut self, event: Event) -> ActionGroup{
        info!("[Bot] Event: {:?}", event);
        vec![]
    }

    fn discord_event(
        &mut self,
        event: DiscordEvent,
        config: &BotConfig,
        queue: &mut EventQueue

    ) -> ActionGroup {
        match event {

            // Server Related Events
            DiscordEvent::ServerCreate(server) => {
                let server_id = Server::id_from_possible_server(&server);
                if config.server_whitelist.contains(&server_id) {
                    let server = Server::from_possible_server(server, config, queue);
                    self.servers.insert(server.id, server);
                }
            },
            DiscordEvent::ServerUpdate(updated_server) => {
                if let Some(server) = self.servers.get_mut(&updated_server.id) {
                    server.update(updated_server);
                }
            },
            DiscordEvent::ServerDelete(_) => {
                warn!("[Bot] Server delete ignored");
            },
            DiscordEvent::ServerMemberAdd(server_id, member) => {
                if let Some(server) = self.servers.get_mut(&server_id) {
                    server.add_member(member, config);
                }
            }
            DiscordEvent::ServerMemberRemove(server_id, user) => {
                if let Some(server) = self.servers.get_mut(&server_id) {
                    server.remove_member_from_user(user);
                }
            },


            // Channel Related Events
            DiscordEvent::ChannelCreate(channel) => {
                let channel = Channel::from_discord_channel(channel);
                if let Some(server_id) = channel.server_id {
                    if let Some(server) = self.servers.get_mut(&server_id) {
                        server.add_channel(channel);
                    }
                }
            },
            DiscordEvent::ChannelUpdate(channel) => {
                let channel = Channel::from_discord_channel(channel);
                if let Some(server_id) = channel.server_id {
                    if let Some(server) = self.servers.get_mut(&server_id) {
                        server.update_channel(channel);
                    }
                }
            },

            DiscordEvent::ChannelDelete(channel) => {
                let channel = Channel::from_discord_channel(channel);
                if let Some(server_id) = channel.server_id {
                    if let Some(server) = self.servers.get_mut(&server_id) {
                        server.remove_channel(channel);
                    }
                }
            },


            // Message Related Events
            DiscordEvent::MessageUpdate { id, channel_id, content, author, .. } => {
                if !author.is_none() && !content.is_none() {
                    return self.message_event(
                        id, channel_id,
                        content.unwrap(), author.unwrap(),
                        Vec::new(),
                        config
                    );
                }
            },

            DiscordEvent::MessageCreate(msg) => {
                return self.message_event(
                    msg.id, msg.channel_id,
                    msg.content, msg.author,
                    msg.attachments,
                    config
                );
            },

            // Connection
            DiscordEvent::Resumed { .. } => {
                for server in self.servers.values_mut() {
                    server.update_voice(queue);
                }
            },

            // Voice
            // Note: Only triggered when actively joining / leaving a voice channel
            DiscordEvent::VoiceServerUpdate { server_id, .. } => {
                if let Some(server_id) = server_id {
                    if let Some(server) = self.servers.get_mut(&server_id) {
                        server.update_voice(queue);
                    }
                }
            },

            DiscordEvent::VoiceStateUpdate(server_id, voice_state) => {
                if let Some(server_id) = server_id {
                    if let Some(server) = self.servers.get_mut(&server_id) {
                        server.update_member_voice_state(
                            voice_state,
                            queue
                        );
                    }
                }
            },

            _ => info!("[Bot] DiscordEvent: {:?}", event)

        }

        vec![]

    }

    fn message_event(
        &mut self,
        id: MessageId, channel_id: ChannelId,
        content: String, author: DiscordUser,
        attachments: Vec<Attachment>,
        config: &BotConfig

    ) -> ActionGroup {

        let mut actions = Vec::new();
        if author.bot {
            info!("[Bot] Ignored bot message.");

        } else if let Some(origin) = self.get_origin_from_message(
            &channel_id,
            &author.id,
        ) {
            info!("[Bot] Received message from whitelisted server.");

            for kind in Message::parse(
                id, author.id, channel_id, content, attachments, origin
            ) {
                actions.extend(match kind {
                    MessageKind::Command(command) => self.command_event(command, config),
                    MessageKind::Upload(upload) => self.upload_event(upload, config)
                })
            }
        }

        actions

    }

    fn command_event(&mut self, command: Command, config: &BotConfig) -> ActionGroup {
        info!("[Bot] Possible {} received", command);
        if let Some((server, member)) = self.get_server_and_member(&command.message) {
            command.parse(server, member, config)

        } else {
            vec![]
        }
    }

    fn upload_event(&mut self, upload: Upload, config: &BotConfig) -> ActionGroup {
        info!("[Bot] Possible {} received", upload);
        if let Some((server, member)) = self.get_server_and_member(&upload.message) {
            upload.process(server, member, config)

        } else {
            vec![]
        }
    }

}


// Internal Utilities ---------------------------------------------------------
impl Bot {

    fn get_origin_from_message(
        &self,
        channel_id: &ChannelId,
        user_id: &UserId

    ) -> Option<(ServerId, MessageOrigin)> {

        for (server_id, server) in &self.servers {
            if server.has_channel(channel_id) {
                return Some((*server_id, MessageOrigin::PublicServerChannel));

            } else if server.has_member(user_id) {
                if self.servers.len() == 1 {
                    return Some((*server_id, MessageOrigin::PrivateServerChannel));

                } else {
                    return Some((*server_id, MessageOrigin::DirectMessage));
                }
            }
        }

        None

    }

    fn get_server_and_member(&self, message: &Message) -> Option<(&Server, &Member)> {
        if let Some(server) = self.servers.get(&message.server_id) {
            if let Some(member) = server.get_member(&message.user_id) {
                Some((server, member))

            } else {
                None
            }

        } else {
            None
        }
    }

}

