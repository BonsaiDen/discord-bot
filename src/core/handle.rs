// Discord Dependencies -------------------------------------------------------
use discord::{ChannelRef, Connection, Discord, State, Error};
use discord::model::{Event, ChannelId, UserId, LiveServer, PrivateChannel};


// Discord State Handle -------------------------------------------------------
pub struct Handle {
    discord: Discord,
    connection: Connection,
    state: State,
    was_updated: bool
}


// Handle Abstractions --------------------------------------------------------
impl Handle {

    pub fn new(token: String) -> Handle {

        let discord = Discord::from_bot_token(&token).expect("Login failed.");
        let (connection, ready) = discord.connect().expect("Connection failed.");
        let state = State::new(ready);

        Handle {
            discord: discord,
            connection: connection,
            state: state,
            was_updated: true
        }

    }

    pub fn recv_event(&mut self) -> Result<Event, Error> {

        match self.connection.recv_event() {

            Ok(event) => {

                self.state.update(&event);

                if let Event::GatewayChanged(data, ready) = event {
                    info!("[State] Gateway changed: {}", data);
                    self.state = State::new(ready);
                    self.was_updated = true;
                    self.recv_event()

                } else {
                    Ok(event)
                }

            }

            Err(err) => {

                warn!("[State] Received error: {:?}", err);

                // WebSocket changed
                if let Error::WebSocket(..) = err {

                    // Handle the websocket connection being dropped
                    let (connection, ready) = self.discord.connect().expect("connect failed");

                    self.connection = connection;
                    self.state = State::new(ready);
                    self.was_updated= true;

                    info!("[State] Reconnected successfully.");

                }

                // Connection closed
                if let Error::Closed(..) = err {
                    return Err(err);
                }

                self.recv_event()

            }

        }

    }

    pub fn was_updated(&mut self) -> bool {
        let updated = self.was_updated;
        self.was_updated = false;
        updated
    }

}


// Getters --------------------------------------------------------------------
impl Handle {

    pub fn user_id(&self) -> UserId {
        self.state.user().id
    }

    pub fn servers(&self) -> &[LiveServer] {
        self.state.servers()
    }

    pub fn find_channel_by_id(&self, channel_id: &ChannelId) -> Option<ChannelRef> {
        self.state.find_channel(channel_id)
    }

    pub fn find_private_channel_for_user(&self, user_id: &UserId) -> Option<PrivateChannel> {
        self.discord.create_private_channel(user_id).ok()
    }

    pub fn send_message_to_user(&self, user_id: &UserId, content: &str) {
        if let Some(channel) = self.find_private_channel_for_user(user_id) {
            self.send_message(&channel.id, content);
        }
    }

    pub fn send_message(&self, channel_id: &ChannelId, content: &str) {
        self.discord.send_message(channel_id, content, "", false).ok();
    }

}

