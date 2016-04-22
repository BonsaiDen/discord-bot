// Discord Dependencies -------------------------------------------------------
use discord::{ChannelRef, Connection, Discord, State, Error};
use discord::model::{
    Event, ChannelId, UserId, ServerId, LiveServer, PrivateChannel
};
use discord::voice::VoiceConnection;


// Internal Dependencies ------------------------------------------------------
use super::{Message, User};


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


// Server Voice ---------------------------------------------------------------
impl Handle {

    pub fn get_server_voice(&mut self, server_id: ServerId) -> &mut VoiceConnection {
        self.connection.voice(server_id)
    }

    pub fn disconnect_server_voice(&mut self, server_id: ServerId){
        self.connection.drop_voice(server_id);
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

    pub fn find_voice_channel_id_for_user(&self, user_id: &UserId) -> Option<ChannelId> {

        let voice_channel = self.state.find_voice_user(*user_id);
        if let Some((_, channel_id)) = voice_channel {
            Some(channel_id)

        } else {
            None
        }

    }

    pub fn find_user_by_id(&self, user_id: &UserId) -> Option<User> {

        for srv in self.state.servers().iter() {
            if let Some(member) = srv.members.iter().find(|m| m.user.id == *user_id) {
                return Some(User::new(&member.user));
            }
        }

        None

    }

    pub fn send_message_to_user(&self, user_id: &UserId, content: &str) {
        if let Some(channel) = self.find_private_channel_for_user(user_id) {
            self.send_message(&channel.id, content);
        }
    }

    pub fn send_message(&self, channel_id: &ChannelId, content: &str) {
        self.discord.send_message(channel_id, content, "", false).ok();
    }

    pub fn delete_message(&self, message: &Message) -> bool {
        if let Err(_) = self.discord.delete_message(message.channel_id, message.id) {
            false

        } else {
            true
        }
    }

}

