// STD Dependencies -----------------------------------------------------------
use std::collections::VecDeque;


// Discord Dependencies -------------------------------------------------------
use discord::voice::VoiceConnection;
use discord::{Connection, Discord, Error, State};
use discord::model::{
    ChannelId, MessageId, UserId, ServerId,
    Event as DiscordEvent,
    PrivateChannel
};


// Low Level Event Abstraction ------------------------------------------------
#[derive(Debug)]
pub enum Event {
    Reconnected,
    Disconnected,
    Received(DiscordEvent),
    SendMessageFailure(ChannelId, String),
    DeleteMessageFailure(ChannelId, MessageId)
}


// Low Level Event Queue ------------------------------------------------------
pub struct EventQueue {
    events: VecDeque<Event>,
    receiver: DiscordHandle
}


// Public Interface -----------------------------------------------------------
impl EventQueue {

    pub fn new(token: String) -> EventQueue {
        EventQueue {
            events: VecDeque::new(),
            receiver: DiscordHandle::from_token(
                token.clone()

            ).expect("[EL] Initial connection failed.")
        }
    }

    pub fn events(&mut self) -> Vec<Event> {

        match self.receiver.recv_event() {
            Ok(event) => {
                self.receiver.update(&event);
                self.events.push_back(Event::Received(event));
            },
            Err(err) => {
                if let Error::WebSocket(..) = err {
                    warn!("[EL] [Receiver] WebSocket closed...");
                    match self.receiver.reconnect() {
                        Ok(r) => {
                            warn!("[EL] [Receiver] Reconnected.!");
                            self.events.push_back(Event::Reconnected);
                            self.receiver = r;
                        },
                        Err(_) => {
                            warn!("[EL] [Receiver] Connection failed!");
                            self.events.push_back(Event::Disconnected);
                        }
                    }

                } else if let Error::Closed(..) = err {
                    warn!("[EL] [Receiver] Connection closed.");
                    self.events.push_back(Event::Disconnected);
                }
            }

        }

        self.events.drain(0..).collect()

    }

    pub fn connect_server_voice<C: FnOnce(&mut VoiceConnection)>(
        &mut self,
        server_id: ServerId,
        channel_id: ChannelId,
        callback: C
    ) {
        let voice_connection = self.receiver.connection.voice(Some(server_id));
        info!("[EL] Create voice connection for Channel#{} on Server#{}", channel_id, server_id);
        voice_connection.connect(channel_id);
        callback(voice_connection);
    }

    pub fn disconnect_server_voice(&mut self, server_id: ServerId) {
        self.receiver.connection.drop_voice(Some(server_id));
    }

    pub fn shutdown(self) {

    }

}

// Message Interface ----------------------------------------------------------
impl EventQueue {

    pub fn send_message_to_user(&mut self, user_id: &UserId, content: String) {
        if let Some(channel) = self.private_channel_for_user(user_id) {
            self.send_message_to_channel(&channel.id, content);
        }
    }

    pub fn send_message_to_channel(&mut self, channel_id: &ChannelId, content: String) {

        if let Err(_) = self.receiver.discord.send_message(channel_id, content.as_str(), "", false) {
            warn!("[EL] Failed to sent message.");
            self.events.push_back(Event::SendMessageFailure(*channel_id, content));

        } else {
            info!("[EL] Message sent.");
        }

    }

    pub fn delete_message(&mut self, message_id: MessageId, channel_id: ChannelId) {
        if let Err(_) = self.receiver.discord.delete_message(&channel_id, &message_id) {
            warn!("[EL] Failed to delete message.");
            self.events.push_back(Event::DeleteMessageFailure(channel_id, message_id));

        } else {
            info!("[EL] Message deleted.");
        }
    }

}


// Internal Interface ---------------------------------------------------------
impl EventQueue {

    fn private_channel_for_user(&self, user_id: &UserId) -> Option<PrivateChannel> {
        self.receiver.discord.create_private_channel(user_id).ok()
    }

}


// Discord Abstraction --------------------------------------------------------
pub struct DiscordHandle {
    token: String,
    discord: Discord,
    connection: Connection,
    state: State
}

impl DiscordHandle {

    fn from_token(token: String) -> Result<DiscordHandle, String> {

        let discord = Discord::from_bot_token(&token).expect("Discord login failed.");
        match discord.connect() {
            Ok((conn, ready)) => Ok(DiscordHandle {
                token: token,
                discord: discord,
                connection: conn,
                state: State::new(ready)
            }),
            Err(err) => Err(err.to_string())
        }

    }

    fn reconnect(&self) -> Result<DiscordHandle, String> {
        DiscordHandle::from_token(self.token.clone())
    }

    fn update(&mut self, event: &DiscordEvent) {
        self.state.update(event);
    }

    fn recv_event(&mut self) -> Result<DiscordEvent, Error> {
        self.connection.recv_event()
    }

}

