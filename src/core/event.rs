// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::sync::{Arc, Mutex};
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
    events: Arc<Mutex<VecDeque<Event>>>,
    sender: DiscordHandle,
    receiver: thread::JoinHandle<()>
}


// Public Interface -----------------------------------------------------------
impl EventQueue {

    pub fn new(token: String) -> EventQueue {

        let events = Arc::new(Mutex::new(VecDeque::new()));
        let receiver = DiscordHandle::from_token(token.clone()).expect("[EL] Initial connection failed.");

        EventQueue {
            events: events.clone(),
            sender: DiscordHandle::from_token(
                token.clone()

            ).expect("[EL] Initial connection failed."),
            receiver: EventQueue::receiver_thread(receiver, events)
        }

    }

    pub fn events(&mut self) -> Vec<Event> {

        let events = if let Ok(mut events) = self.events.lock() {
            events.drain(0..).collect()

        } else {
            vec![]
        };

        for e in &events {
            match e {
                &Event::Received(ref e) => self.sender.update(e),
                &Event::Reconnected => {
                    self.sender =self.sender.reconnect().expect("[EL] Sender reconnect failed!");
                },
                _ => {}
            }
        }

        events

    }

    pub fn connect_server_voice<C: FnOnce(&mut VoiceConnection)>(
        &mut self,
        server_id: ServerId,
        channel_id: ChannelId,
        callback: C
    ) {
        let voice_connection = self.sender.connection.voice(Some(server_id));
        info!("[EL] Create voice connection for {} on {}", channel_id, server_id);
        voice_connection.connect(channel_id);
        callback(voice_connection);
    }

    pub fn disconnect_server_voice(&mut self, server_id: ServerId) {
        self.sender.connection.drop_voice(Some(server_id));
    }

    pub fn shutdown(self) {
        self.receiver.join().ok();
    }

}

// Message Interface ----------------------------------------------------------
impl EventQueue {

    pub fn send_message_to_user(&self, user_id: &UserId, content: String) {
        if let Some(channel) = self.private_channel_for_user(user_id) {
            self.send_message_to_channel(&channel.id, content);
        }
    }

    pub fn send_message_to_channel(&self, channel_id: &ChannelId, content: String) {

        if let Err(_) = self.sender.discord.send_message(channel_id, content.as_str(), "", false) {
            warn!("[EL] Failed to sent message.");
            if let Ok(mut events) = self.events.lock() {
                events.push_back(Event::SendMessageFailure(*channel_id, content));
            }

        } else {
            info!("[EL] Message sent.");
        }

    }

    pub fn delete_message(&self, message_id: MessageId, channel_id: ChannelId) {
        if let Err(_) = self.sender.discord.delete_message(&channel_id, &message_id) {
            warn!("[EL] Failed to delete message.");
            if let Ok(mut events) = self.events.lock() {
                events.push_back(Event::DeleteMessageFailure(channel_id, message_id));
            }

        } else {
            info!("[EL] Message deleted.");
        }
    }

}


// Internal Interface ---------------------------------------------------------
impl EventQueue {

    fn receiver_thread(
        mut receiver: DiscordHandle,
        events: Arc<Mutex<VecDeque<Event>>>

    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {

            loop {
                match receiver.recv_event() {
                    Ok(event) => {

                        receiver.update(&event);

                        if let Ok(mut queue) = events.lock() {
                            queue.push_back(Event::Received(event));
                        }

                    },
                    Err(err) => {
                        if let Error::WebSocket(..) = err {
                            warn!("[EL] [Receiver] WebSocket closed...");
                            match receiver.reconnect() {
                                Ok(r) => {
                                    warn!("[EL] [Receiver] Reconnected.!");
                                    if let Ok(mut queue) = events.lock() {
                                        queue.push_back(Event::Reconnected);
                                    }
                                    receiver = r;
                                },
                                Err(_) => {
                                    warn!("[EL] [Receiver] Connection failed!");
                                    if let Ok(mut queue) = events.lock() {
                                        queue.push_back(Event::Disconnected);
                                    }
                                    break;
                                }
                            }

                        } else if let Error::Closed(..) = err {
                            warn!("[EL] [Receiver] Connection closed.");
                            if let Ok(mut queue) = events.lock() {
                                queue.push_back(Event::Disconnected);
                            }
                            break;
                        }
                    }
                }
            }

        })
    }

    fn private_channel_for_user(&self, user_id: &UserId) -> Option<PrivateChannel> {
        self.sender.discord.create_private_channel(user_id).ok()
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

