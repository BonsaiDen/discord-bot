// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId, VoiceState};


// Internal Dependencies ------------------------------------------------------
use super::User;
use super::voice::Greeting;


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    // General
    id: ServerId,
    name: String,
    channel_count: usize,
    member_count: usize,

    // Voice
    voice_states: Vec<(ChannelId, User)>,
    voice_greetings: HashMap<String, Greeting>

}

impl Server {

    pub fn new(id: ServerId) -> Server {
        Server {

            // General
            id: id,
            name: "".to_string(),
            channel_count: 0,
            member_count: 0,

            // Voice
            voice_states: Vec::new(),
            voice_greetings: HashMap::new()

        }
    }

}

// Voice Handling --------------------------------------------------------------
impl Server {

    pub fn initialize_voices(&mut self) {

        info!("[Server] [{}] [Voice] Initializing.", self);

        // TODO re-join last connected voice channel

    }

    pub fn update_voice(&mut self, voice: VoiceState, user: User) {

        if let Some(channel_id) = voice.channel_id {
            if self.voice_states.iter().any(|&(_, ref u)| u.id == user.id) {
                info!("[Server] [{}] [{}] [Voice] Switched voice channel.", self, user);

            } else {
                info!("[Server] [{}] [{}] [Voice] Joined voice channel.", self, user);
                self.add_voice_state(channel_id, user);
            }

        } else {
            info!("[Server] [{}] [{}] [Voice] Left voice channel.", self, user);
            self.voice_states.retain(|&(_, ref u)| u.id != user.id);
        }

        // Leave the voice channel if it becomes empty
        //if let Some(channel) = self.connection.voice(server_id).current_channel() {
        //    if let Some(srv) = self.state.servers().iter().find(|srv| srv.id == server_id) {
        //        if srv.voice_states.iter().filter(|vs| vs.channel_id == Some(channel)).count() <= 1 {
        //            info!("[Audio] [Info] Leaving empty Voice Channel");
        //            self.connection.drop_voice(server_id);
        //        }
        //    }
        //}

    }

}

// Setters --------------------------------------------------------------------
impl Server {

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn clear_channels(&mut self) {
        self.channel_count = 0;
    }

    pub fn inc_channels(&mut self) {
        self.channel_count += 1;
    }

    pub fn dec_channels(&mut self) {
        self.channel_count -= 1;
    }

    pub fn clear_members(&mut self) {
        self.member_count = 0;
    }

    pub fn inc_members(&mut self) {
        self.member_count += 1;
    }

    pub fn dec_members(&mut self) {
        self.member_count -= 1;
    }

    pub fn clear_voice_states(&mut self) {
        self.voice_states.clear();
    }

    pub fn add_voice_state(&mut self, channel_id: ChannelId, user: User) {
        info!("[Server] [{}] [{}] [Voice] State added.", self, user);
        self.voice_states.push((channel_id, user));
    }

}


// Getters --------------------------------------------------------------------
impl Server {

    pub fn id(&self) -> &ServerId {
        &self.id
    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

