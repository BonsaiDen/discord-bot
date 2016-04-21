// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{Event, Channel, ChannelId, UserId, ServerId};


// Modules --------------------------------------------------------------------
mod command;
mod handle;
mod message;
mod server;
mod user;


// Internal Dependencies ------------------------------------------------------
pub use self::handle::Handle;
use self::message::Message;
pub use self::server::Server;
pub use self::user::User;


// Bot Abstraction ------------------------------------------------------------
pub struct Bot {

    handle: Option<Handle>,

    // Whitelisting
    server_whitelist: Option<Vec<ServerId>>,

    // Internal State
    server_states: HashMap<ServerId, Server>,
    channel_map: HashMap<ChannelId, ServerId>,
    user_map: HashMap<UserId, Vec<ServerId>>

}


// Connection Handling --------------------------------------------------------
impl Bot {

    pub fn new(token: String, server_whitelist: Option<Vec<ServerId>>) -> Bot {

        Bot {

            handle: Some(Handle::new(token)),

            // Whitelisting
            server_whitelist: server_whitelist,

            // Internal State
            server_states: HashMap::new(),
            channel_map: HashMap::new(),
            user_map: HashMap::new()

        }

    }

    pub fn connect(&mut self) {
        self.event_loop();
    }

}


// Events and Messages --------------------------------------------------------
impl Bot {

    fn event_loop(&mut self) {

        let mut handle = self.handle.take().unwrap();

        loop {

            match handle.recv_event() {
                Ok(event) => {

                    self.handle_event(&mut handle, event);

                    if handle.was_updated() {
                        info!("[State] Connection updated.");
                        self.init_servers_and_channels(&handle);
                    }

                }
                Err(_) => {
                    info!("[State] Connection closed.");
                    break;
                }
            }

        }

    }

    fn handle_event(&mut self, handle: &mut Handle, event: Event) {
        match event {

            Event::ServerUpdate(srv) => {
                if let Some(server) = self.get_server(&srv.id) {
                    server.name = srv.name.to_string();
                }
            }

            Event::ChannelCreate(channel) => {
                if let Channel::Public(channel) = channel {
                    if self.is_whitelisted_server(&channel.server_id) {
                        info!("[State] Mapped channel {}({}) -> {}", channel.name, channel.id.0, channel.server_id.0);
                        self.channel_map.insert(channel.id, channel.server_id);
                        if let Some(server) = self.get_server(&channel.server_id) {
                            server.channel_count += 1;
                        }
                    }
                }
            }

            Event::ChannelDelete(channel) => {
                if let Channel::Public(channel) = channel {
                    if self.is_whitelisted_server(&channel.server_id) {
                        info!("[State] Unmapped channel {}", channel.id.0);
                        self.channel_map.remove(&channel.id);
                        if let Some(server) = self.get_server(&channel.server_id) {
                            server.channel_count -= 1;
                        }
                    }
                }
            }

            Event::ServerMemberAdd(server_id, member) => {
                if self.is_whitelisted_server(&server_id) {

                    info!("[State] Mapped user {}({}) -> {}", member.user.name, member.user.id.0, server_id.0);

                    if let Some(server) = self.get_server(&server_id) {
                        server.member_count += 1;
                    }

                    let server_list = self.user_map.entry(member.user.id).or_insert_with(|| Vec::new());
                    if !server_list.contains(&server_id) {
                        server_list.push(server_id);
                    }


                }
            }

            Event::ServerMemberRemove(server_id, user) => {
                if self.is_whitelisted_server(&server_id) {

                    info!("[State] Unmapped user {:?}", user.id);

                    if let Some(server_list) = self.user_map.get_mut(&user.id) {
                        server_list.retain(|id| {
                            *id != server_id
                        });
                    }

                    if let Some(server) = self.get_server(&server_id) {
                        server.member_count -= 1;
                    }

                }
            }

            Event::MessageUpdate { id, channel_id, author, content, .. } => {
                if !author.is_none() && !content.is_none() {

                    let author = User::new(&author.unwrap());

                    if let Some((server_id, unique)) = self.server_id_for_channel_or_user(
                        &channel_id, &author
                    ) {

                        let message = Message {
                            id: &id,
                            server_id: &server_id,
                            channel_id: &channel_id,
                            author: &author,
                            content: &content.unwrap(),
                            was_edited: true
                        };

                        message.handle(
                            handle,
                            self.get_server(&server_id).unwrap(),
                            unique
                        );


                    } else {
                        info!("[Event] Message edit from non-whitelisted server.");
                    }

                }
            }

            Event::MessageCreate(msg) => {

                let author = User::new(&msg.author);

                if let Some((server_id, unique)) = self.server_id_for_channel_or_user(
                    &msg.channel_id, &author
                ) {

                    let message = Message {
                        id: &msg.id,
                        server_id: &server_id,
                        channel_id: &msg.channel_id,
                        author: &author,
                        content: &msg.content,
                        was_edited: false
                    };

                    message.handle(
                        handle,
                        self.get_server(&server_id).unwrap(),
                        unique
                    );

                } else {
                    info!("[Event] Message from non-whitelisted server.");
                }

            }

            Event::VoiceStateUpdate(_, _) => {
                //self.update_voice(server_id, voice_state);
            }

            Event::Unknown(name, data) => {
                debug!("[Unknown Event] {}: {:?}", name, data);
            }

            _ => {}

        }
    }

}


// Servers and Channels -------------------------------------------------------
impl Bot {

    fn init_servers_and_channels(&mut self, handle: &Handle) {

        let mut channels_to_map = Vec::new();
        let mut users_to_map = Vec::new();
        let mut valid_servers = Vec::new();

        for srv in handle.servers() {

            // Get or create state for whitelisted servers
            if let Some(server) = self.get_server(&srv.id) {

                // Server Update
                server.name = srv.name.to_string();

                // Mark this server as valid
                valid_servers.push(srv.id);

                // Push all of the servers current channels for mappingJ
                server.channel_count = 0;
                for channel in &srv.channels {
                    server.channel_count += 1;
                    channels_to_map.push((
                        channel.id, srv.id, channel.name.to_string())
                    );
                }

                server.member_count = 0;
                for member in &srv.members {
                    server.member_count += 1;
                    users_to_map.push((
                        member.user.id, srv.id, member.user.name.to_string())
                    );
                }

                info!("[State] Mapped server {}", server);

            }

        }

        // Insert new channels to map
        for (channel_id, server_id, channel_name) in channels_to_map {
            if !self.channel_map.contains_key(&channel_id) {
                info!("[State] Mapped channel {}({}) -> {}", channel_name, channel_id.0, server_id.0);
                self.channel_map.insert(channel_id, server_id);
            }
        }

        // Insert new users to map
        for (user_id, server_id, user_name) in users_to_map {

            let server_list = self.user_map.entry(user_id).or_insert_with(|| Vec::new());
            if !server_list.contains(&server_id) {
                info!("[State] Mapped user {}({}) -> {}", user_name, user_id.0, server_id.0);
                server_list.push(server_id);
            }

        }

        // Check for now invalid servers and remove them
        let invalid_servers = self.server_states.values().filter(|s| {
            !valid_servers.contains(s.id())

        }).map(|s| s.id().clone()).collect::<Vec<ServerId>>();

        for server_id in invalid_servers {
            info!("[State] Unmapped server {}", server_id.0);
            self.server_states.remove(&server_id);
        }

        // Also remove any channels mapped to them
        let invalid_channels = self.channel_map.iter().filter(|&(_, s)| {
            !valid_servers.contains(&s)

        }).map(|(c, _)| c.clone()).collect::<Vec<ChannelId>>();

        for channel_id in invalid_channels {
            info!("[State] Unmapped channel {}", channel_id.0);
            self.channel_map.remove(&channel_id);
        }

        // Remove the invalid servers from our user mappings
        for (_, server_list) in &mut self.user_map {
            server_list.retain(|server_id| {
                valid_servers.contains(&server_id)
            })
        }

        // Remove all users without any mapped server
        let invalid_users = self.user_map.iter().filter(|&(_, server_list)| {
            server_list.is_empty()

        }).map(|(c, _)| c.clone()).collect::<Vec<UserId>>();

        for user_id in invalid_users {
            info!("[State] Unmapped user {}", user_id.0);
            self.user_map.remove(&user_id);
        }

    }

    fn get_server(&mut self, server_id: &ServerId) -> Option<&mut Server> {

        if self.is_whitelisted_server(server_id) {
            Some(self.server_states.entry(*server_id).or_insert_with(|| {
                Server::new(*server_id)
            }))

        } else {
            None
        }

    }

    fn server_id_for_channel_or_user(
        &self,
        channel_id: &ChannelId,
        user: &User

    ) -> Option<(ServerId, bool)> {

        if let Some(server_id) = self.channel_map.get(channel_id) {
            if self.is_whitelisted_server(server_id) {
                Some((server_id.clone(), true))

            } else {
                None
            }

        } else if let Some(server_list) = self.user_map.get(&user.id) {
            if server_list.is_empty() {
                None

            } else {
                Some((
                    server_list.get(0).unwrap().clone(),
                    server_list.len() == 1
                ))
            }

        } else {
            None
        }

    }

    fn is_whitelisted_server(&self, server_id: &ServerId) -> bool {
        match self.server_whitelist {
            Some(ref list) => list.iter().any(|id| id == server_id),
            None => true
        }
    }

}

