// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;
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
use action::ActionGroup;
use audio::MixerEvent;
use server::Server;
use core::{
    Channel,
    Event, EventQueue,
    Member,
    Message, MessageContent
};


// Bot Configuration ----------------------------------------------------------
pub struct BotConfig {
    pub bot_nickname: String,
    pub server_whitelist: Vec<ServerId>,
    pub config_path: PathBuf,
    pub effect_playback_separation_ms: u64,
    pub greeting_separation_ms: u64,
    pub flac_max_file_size: u64,
    pub flac_sample_rate: u32,
    pub flac_bits_per_sample: u8
}

impl Default for BotConfig {
    fn default() -> BotConfig {
        BotConfig {
            bot_nickname: "".to_string(),
            server_whitelist: Vec::new(),
            config_path: PathBuf::from(""),
            effect_playback_separation_ms: 0,
            greeting_separation_ms: 0,
            flac_max_file_size: 0,
            flac_sample_rate: 0,
            flac_bits_per_sample: 0
        }
    }
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
                    Event::Received(event) => {
                        self.discord_event(event, &config, &mut queue)
                    },
                    Event::Timer => {
                        self.timer_event(&config, &mut queue)
                    },
                    _ => self.event(event)
                };

                // Run resulting Actions
                let mut delayed_actions = vec![];
                while !actions.is_empty() {

                    let mut next_actions = Vec::new();
                    for mut action in actions.drain(0..) {
                        if action.ready() {
                            info!("[Bot] Running {}...", action);
                            next_actions.extend(
                                action.run(&mut self, &config, &mut queue)
                            );

                        } else {
                            delayed_actions.push(action);
                        }
                    }

                    actions.extend(next_actions);

                }

                actions.extend(delayed_actions);

            }

        }

    }

}


// Event Handling -------------------------------------------------------------
impl Bot {

    fn event(&mut self, event: Event) -> ActionGroup {
        info!("[Bot] Event: {:?}", event);
        vec![]
    }

    fn discord_event(
        &mut self,
        event: Box<DiscordEvent>,
        config: &BotConfig,
        queue: &mut EventQueue

    ) -> ActionGroup {
        match *event {

            // Server Related Events
            DiscordEvent::ServerCreate(server) => {

                let server_id = Server::id_from_possible_server(&server);

                if self.servers.contains_key(&server_id) {
                    // In case of a client reconnect, also reconnect any existing
                    // voice connections
                    if let Some(mut server) = self.servers.get_mut(&server_id) {
                        server.reconnect_voice(queue);
                    }

                } else if config.server_whitelist.contains(&server_id) {
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
            DiscordEvent::MessageUpdate { id, channel_id, ref content, ref author, .. } => {
                if let Some(ref content) = *content {
                    if let Some(ref author) = *author {
                        return self.message_event(
                            id, channel_id,
                            content,
                            author,
                            Vec::new(),
                            config
                        );
                    }
                }
            },

            DiscordEvent::MessageCreate(msg) => {
                return self.message_event(
                    msg.id, msg.channel_id,
                    &msg.content,
                    &msg.author,
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
                        return server.update_member_voice_state(
                            voice_state,
                            queue,
                            config
                        )
                    }
                }
            },

            _ => { /* ignore all other events*/ }

        }

        vec![]

    }

    fn timer_event(
        &mut self,
        config: &BotConfig,
        queue: &mut EventQueue

    ) -> ActionGroup {

        // Fetch mixer events from all servers
        let events: Vec<MixerEvent> = self.servers.values().map(|server| {
            server.events()

        }).flat_map(|e| e).collect();

        events.into_iter().flat_map(|event| {
            self.mixer_event(event, config, queue)

        }).collect()

    }

    fn mixer_event(
        &mut self,
        event: MixerEvent,
        _: &BotConfig,
        _: &mut EventQueue

    ) -> ActionGroup {
        match event {
            MixerEvent::Completed(effect, mut action) => {
                info!("[Bot] MixerEvent effect playback completed: {:?}", effect);
                if let Some(action) = action.take() {
                    vec![action]

                } else {
                    vec![]
                }
            },
            MixerEvent::Canceled(effect, _) => {
                info!("[Bot] MixerEvent effect playback canceled: {:?}", effect);
                vec![]
            }
        }
    }

    fn message_event(
        &mut self,
        id: MessageId, channel_id: ChannelId,
        content: &str, author: &DiscordUser,
        attachments: Vec<Attachment>,
        bot_config: &BotConfig

    ) -> ActionGroup {

        if author.bot {
            info!("[Bot] Ignored message from a bot.");
            vec![]

        } else if let Some((server_id, is_unique_server)) = self.get_server_for_channel(
            &channel_id,
            &author.id
        ) {
            info!("[Bot] Parsing message from white listed server...");

            let message = Message::from_parts(
                id,
                author.id,
                channel_id,
                server_id,
                is_unique_server
            );

            message.parse_contents(content, attachments).into_iter().flat_map(|content| {
                self.parse_content(content, bot_config)

            }).collect()

        } else {
            vec![]
        }

    }

    fn parse_content(
        &self,
        content: MessageContent,
        bot_config: &BotConfig

    ) -> ActionGroup {
        match content {
            MessageContent::Command(name, arguments, message) => {
                if let Some((
                    server,
                    member

                )) = self.get_server_and_member(&message) {
                    Command::from_parts(
                        name, arguments, message,
                        server, member, bot_config

                    ).process()

                } else {
                    vec![]
                }
            },
            MessageContent::Upload(attachment, message) => {
                if let Some((
                    _,
                    member

                )) = self.get_server_and_member(&message) {
                    Upload::from_message(attachment, message)
                           .process(member, bot_config)

                } else {
                    vec![]
                }
            }
        }
    }

}


// Internal Utilities ---------------------------------------------------------
impl Bot {

    fn get_server_for_channel(
        &self,
        channel_id: &ChannelId,
        user_id: &UserId

    ) -> Option<(ServerId, bool)> {

        for (server_id, server) in &self.servers {
            if server.has_channel(channel_id) {
                return Some((*server_id, true));

            } else if server.has_member(user_id) {
                if self.servers.len() == 1 {
                    return Some((*server_id, true));

                } else {
                    return Some((*server_id, false));
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

