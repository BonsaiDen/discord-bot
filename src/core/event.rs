// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::time::Duration;
use std::iter::Iterator;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};


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
    Tick,
    Reconnected,
    Disconnected,
    Received(DiscordEvent),
    SendMessage(ChannelId, String),
    SendMessageFailure(ChannelId, String),
    DeleteMessage(ChannelId, MessageId),
    DeleteMessageFailure(ChannelId, MessageId)
}


// Event Iterator for Receive Synchronization ---------------------------------
pub struct EventIterator<'a> {
    queue: MutexGuard<'a, VecDeque<Event>>
}

impl<'a> EventIterator<'a> {

    fn new(queue: MutexGuard<'a, VecDeque<Event>>) -> EventIterator<'a> {
        EventIterator {
            queue: queue
        }
    }

}

impl<'a> Iterator for EventIterator<'a> {

    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }

}


// Low Level Event Queue ------------------------------------------------------
pub struct EventQueue {

    discord: Arc<Mutex<Discord>>,
    connection: Arc<Mutex<Option<Connection>>>,
    state: Arc<Mutex<Option<State>>>,

    receiver_thread: Option<thread::JoinHandle<()>>,
    receiver_event_queue: Arc<Mutex<VecDeque<Event>>>,

    sender_thread: Option<thread::JoinHandle<()>>,
    sender_event_queue: Option<Sender<Option<Event>>>

}


// Public Interface -----------------------------------------------------------
impl EventQueue {

    pub fn new(token: String) -> EventQueue {
        EventQueue {

            discord: Arc::new(Mutex::new(
                Discord::from_bot_token(&token).expect("[EL] Discord login failed.")
            )),
            connection: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(None)),

            receiver_thread: None,
            receiver_event_queue: Arc::new(Mutex::new(VecDeque::new())),

            sender_thread: None,
            sender_event_queue: None

        }
    }

    pub fn connect(&mut self) {
        info!("[EL] Connecting...");
        self.sender_thread = Some(self.create_sender());
        self.receiver_thread = Some(self.create_receiver());
        info!("[EL] Connected!");
    }

    pub fn events(&self) -> EventIterator {
        // Return a iterator which holds a lock on the event queue and prevents
        // the receiver_thread from holding a lock on internal connection object
        // while messages are being process and the bot logic is run
        EventIterator::new(
            self.receiver_event_queue.lock().expect("[EL] Failed to lock receiver event queue")
        )
    }

    pub fn connect_server_voice<C: FnOnce(&mut VoiceConnection)>(
        &self,
        server_id: ServerId,
        channel_id: ChannelId,
        callback: C
    ) {
        if let Ok(mut connection) = self.connection.lock() {
            if let Some(connection) = connection.as_mut() {
                let voice_connection = connection.voice(Some(server_id));
                info!("[EL] Create voice connection for {} on {}", channel_id, server_id);
                voice_connection.connect(channel_id);
                callback(voice_connection);
            }

        } else {
            error!("[EL] Failed to get connect lock for server voice connect.");
        }
    }

    pub fn disconnect_server_voice(&self, server_id: ServerId) {
        if let Ok(mut connection) = self.connection.lock() {
            if let Some(connection) = connection.as_mut() {
                connection.drop_voice(Some(server_id));
            }

        } else {
            error!("[EL] Failed to get connect lock for server voice disconnect.");
        }
    }

    pub fn shutdown(&mut self) {
        // Note: right now this will only shutdown the sender
        if let Some(ref queue) = self.sender_event_queue {
            queue.send(None).ok();
            self.sender_thread.take().unwrap().join().ok();
            self.receiver_thread.take().unwrap().join().ok();
        }
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
        if let Some(ref queue) = self.sender_event_queue {
            queue.send(
                Some(Event::SendMessage(*channel_id, content))

            ).ok();
        }
    }

    pub fn delete_message(&self, message_id: MessageId, channel_id: ChannelId) {
        if let Some(ref queue) = self.sender_event_queue {
            queue.send(
                Some(Event::DeleteMessage(channel_id, message_id))

            ).ok();
        }
    }

}


// Internal Interface ---------------------------------------------------------
impl EventQueue {

    fn create_receiver(&mut self) -> thread::JoinHandle<()> {

        info!("[EL] Creating Receiver...");

        let discord = self.discord.clone();
        let connection = self.connection.clone();
        let state = self.state.clone();
        let receiver_event_queue = self.receiver_event_queue.clone();

        thread::spawn(move || {
            info!("[EL/RT] Created!");
            receiver_loop(discord, connection, state, receiver_event_queue);
        })

    }

    fn create_sender(&mut self) -> thread::JoinHandle<()> {

        info!("[EL] Creating Sender...");

        let discord = self.discord.clone();
        let receiver_event_queue = self.receiver_event_queue.clone();
        let (send_queue, recv_queue) = channel::<Option<Event>>();

        self.sender_event_queue = Some(send_queue);

        thread::spawn(move || {
            info!("[EL/ST] Created!");
            sender_loop(discord, recv_queue, receiver_event_queue);
        })

    }

    fn private_channel_for_user(&self, user_id: &UserId) -> Option<PrivateChannel> {
        if let Ok(discord) = self.discord.lock() {
            discord.create_private_channel(user_id).ok()

        } else {
            warn!("[EV] Failed to get discord lock for creating private channel.");
            None
        }
    }

}


// Helpers --------------------------------------------------------------------
fn receiver_loop(
    discord: Arc<Mutex<Discord>>,
    connection: Arc<Mutex<Option<Connection>>>,
    state: Arc<Mutex<Option<State>>>,
    receiver_event_queue: Arc<Mutex<VecDeque<Event>>>
) {

    // TODO support closing from the outside?
    // We would need a way to unblock the connection.recv_event() call
    let mut is_connected = false;
    loop {

        if is_connected {

            // Check if queue is empty, but don't hold a lock afterwards
            let is_empty = if let Ok(queue) = receiver_event_queue.try_lock() {
                queue.is_empty()

            } else {
                false
            };

            if is_empty {
                // Only fetch new events once queued got emptied by external threads
                match connection.lock().unwrap().as_mut().unwrap().recv_event() {

                    Ok(event) => {
                        //info!("[EL/RT] Event received.");
                        state.lock().unwrap().as_mut().unwrap().update(&event);
                        if let Ok(mut queue) = receiver_event_queue.lock() {
                            queue.push_back(Event::Received(event));
                        }
                    },

                    Err(err) => {
                        if let Error::WebSocket(..) = err {
                            warn!("[EL/RT] WebSocket closed.");
                            is_connected = false;
                        }

                        if let Error::Closed(..) = err {
                            warn!("[EL/RT] Connection closed.");
                            if let Ok(mut queue) = receiver_event_queue.lock() {
                                queue.push_back(Event::Disconnected);
                            }
                            break;
                        }
                    }

                }
            }

            // Give other threads a chance to lock and empty the receiver
            // queue
            thread::sleep(Duration::from_millis(25));

        // (Re)-Connect if required
        } else if let Ok(discord) = discord.lock() {

            info!("[EL/RT] Reconnecting...");

            if let Ok((conn, ready)) = discord.connect() {

                *connection.lock().unwrap() = Some(conn);
                *state.lock().unwrap() = Some(State::new(ready));
                is_connected = true;

                info!("[EL/RT] Reconnected!");

                if let Ok(mut queue) = receiver_event_queue.lock() {
                    queue.push_back(Event::Reconnected);
                }

            } else if let Ok(mut queue) = receiver_event_queue.lock() {
                error!("[EL/RT] Reconnect failed.");
                queue.push_back(Event::Disconnected);
                break;
            }

        }

    }

}

fn sender_loop(
    discord: Arc<Mutex<Discord>>,
    recv_queue: Receiver<Option<Event>>,
    receiver_event_queue: Arc<Mutex<VecDeque<Event>>>
) {

    let mut events = Vec::new();
    let mut accumulated = 0;
    loop {

        if let Ok(message) = recv_queue.try_recv() {
            if let Some(message) = message {

                if let Ok(discord) = discord.lock() {
                    match message {

                        Event::SendMessage(cid, content) => {
                            if let Err(_) = discord.send_message(
                                &cid,
                                content.as_str(),
                                "",
                                false

                            ) {
                                warn!("[EL/ST] Failed to sent message.");
                                events.push(Event::SendMessageFailure(cid, content));

                            } else {
                                info!("[EL/ST] Message sent.");
                            }
                        },

                        Event::DeleteMessage(cid, mid) => {
                            if let Err(_) = discord.delete_message(&cid, &mid) {
                                warn!("[EL/ST] Failed to delete message.");
                                events.push(Event::DeleteMessageFailure(cid, mid));

                            } else {
                                info!("[EL/ST] Message deleted.");
                            }
                        },

                        _ => panic!("[EL/ST] Unexpected message type fed into sender.")

                    }
                }

            } else {
                break
            }
        }

        // Tick Events
        accumulated += 50;

        if accumulated >= 1000 {

            accumulated -= 1000;
            events.push(Event::Tick);

            if let Ok(mut queue) = receiver_event_queue.try_lock() {
                for e in events.drain(0..) {
                    queue.push_back(e);
                }
            }

        }

        thread::sleep(Duration::from_millis(50));

    }

}

