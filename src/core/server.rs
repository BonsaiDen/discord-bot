// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::atomic::Ordering;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId, VoiceState};


// Internal Dependencies ------------------------------------------------------
use super::{Handle, User, Effect, EffectManager};
use super::voice::{
    Greeting,
    Listener, ListenerCount, EmptyListenerCount,
    Mixer,
    Queue, QueueHandle, QueueEntry, EmptyQueue
};


// Statics --------------------------------------------------------------------
static PLAY_EFFECT_JOIN_DELAY: usize = 300;


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    // General
    id: ServerId,
    name: String,
    channel_count: usize,
    member_count: usize,

    // Voice
    last_voice_channel: Option<ChannelId>,
    voice_listener_handle: Option<QueueHandle>,
    voice_listener_count: ListenerCount,
    voice_states: Vec<(ChannelId, User)>,
    voice_greetings: HashMap<String, Greeting>,
    voice_queue: Queue,

    // Effects
    effect_manager: EffectManager

}

impl Server {

    pub fn new(id: ServerId, effects_directory: PathBuf) -> Server {

        Server {

            // General
            id: id,
            name: "".to_string(),
            channel_count: 0,
            member_count: 0,

            // Voice
            last_voice_channel: None,
            voice_listener_handle: None,
            voice_listener_count: EmptyListenerCount::create(),
            voice_states: Vec::new(),
            voice_greetings: HashMap::new(),
            voice_queue: EmptyQueue::create(),

            // Effects
            effect_manager: EffectManager::new(effects_directory)

        }

    }

}

// Effect Playback ------------------------------------------------------------
impl Server {

    pub fn play_effects(
        &mut self,
        handle: &mut Handle,
        channel_id: ChannelId,
        effects: Vec<Effect>,
        immediate: bool,
        mut delay: usize
    ) {

        // Add additional delay if we need to join the channel
        if self.join_voice_channel(handle, Some(channel_id)) {
            delay += PLAY_EFFECT_JOIN_DELAY;
        }

        if let Ok(mut queue) = self.voice_queue.lock() {
            if immediate {
                info!("[Server] [{}] [Voice] {} effect(s) added for immediate playback in {}ms.", self, effects.len(), delay);
                queue.push_back(QueueEntry::EffectList(effects, delay));

            } else {
                info!("[Server] [{}] [Voice] {} effect(s) added for queued playback in {}ms.", self, effects.len(), delay);
                queue.push_back(QueueEntry::QueuedEffectList(effects, delay));
            }
        }

    }

    pub fn request_silence(&mut self) {
        if let Ok(mut queue) = self.voice_queue.lock() {
            queue.clear();
            queue.push_back(QueueEntry::Reset);
        }
    }

    pub fn map_effects(&mut self, list: &[String]) -> Vec<Effect> {
        self.effect_manager.map_from_patterns(list)
    }

}


// Voice Handling -------------------------------------------------------------
impl Server {

    pub fn initialize_voices(&mut self, handle: &mut Handle) {
        info!("[Server] [{}] [Voice] Initializing.", self);
        self.join_voice_channel(handle, None);
    }

    pub fn update_voice(&mut self, handle: &mut Handle, voice: VoiceState, user: User) {

        if user.id == handle.user_id() {
            if let Some(_) = voice.channel_id {
                info!("[Server] [{}] [Voice] Joined channel.", self);

            } else {
                info!("[Server] [{}] [Voice] Left channel.", self);
                self.voice_listener_handle = None;
            }

        } else if let Some(channel_id) = voice.channel_id {
            if self.voice_states.iter().any(|&(_, ref u)| u.id == user.id) {
                if self.update_voice_state(channel_id, voice, &user) {
                    info!("[Server] [{}] [{}] [Voice] User switched channel.", self, user);

                } else {
                    info!("[Server] [{}] [{}] [Voice] User state updated.", self, user);
                }

            } else {
                info!("[Server] [{}] [{}] [Voice] User joined channel.", self, user);
                self.add_voice_state(channel_id, &voice, user);
            }

        } else {
            info!("[Server] [{}] [{}] [Voice] User left channel.", self, user);
            self.voice_states.retain(|&(_, ref u)| u.id != user.id);
        }

        if let Some(channel_id) = handle.get_server_voice(self.id).current_channel() {

            // TODO clean up

            // Update Active Listener Count
            let active_listener_count = self.voice_states.iter().filter(|&&(ref id, ref user)| {
                *id == channel_id && !user.mute && !user.deaf

            }).count();

            self.voice_listener_count.store(active_listener_count, Ordering::Relaxed);

            // Check channel user count and leave if empty
            let channel_user_count = self.voice_states.iter().filter(|&&(ref id, _)| {
                *id == channel_id

            }).count();

            if channel_user_count == 0 {
                info!("[Server] [{}] [Voice] Leaving empty channel.", self);
                handle.disconnect_server_voice(self.id)
            }

        }

    }

    pub fn join_voice_channel(&mut self, handle: &mut Handle, channel_id: Option<ChannelId>) -> bool {

        if let Some(target_id) = channel_id.or(self.last_voice_channel) {

            if self.last_voice_channel.is_none() || self.voice_listener_handle.is_none() {
                info!("[Server] [{}] [Voice] Joining channel.", self);
                self.init_voice_connection(handle, target_id);
                true

            } else if channel_id.is_none() {
                info!("[Server] [{}] [Voice] Re-joining channel.", self);
                self.init_voice_connection(handle, target_id);
                true

            } else if channel_id == self.last_voice_channel {
                info!("[Server] [{}] [Voice] Already in channel.", self);
                false

            } else {
                info!("[Server] [{}] [Voice] Switching channel.", self);
                self.init_voice_connection(handle, target_id);
                true
            }

        } else {
            false
        }

    }

    fn update_voice_state(&mut self, channel_id: ChannelId, voice: VoiceState, user: &User) -> bool {
        if let Some(&mut(ref mut channel, ref mut user)) = self.voice_states.iter_mut().find(|&&mut (_, ref u)| u.id == user.id) {

            user.deaf = voice.deaf || voice.self_deaf;
            user.mute = voice.mute || voice.self_mute;

            if *channel == channel_id {
                false

            } else {
                *channel = channel_id;
                true
            }

        } else {
            false
        }
    }

    fn init_voice_connection(&mut self, handle: &mut Handle, channel_id: ChannelId) {

        let voice_connection = handle.get_server_voice(self.id);
        voice_connection.connect(channel_id);

        match self.voice_listener_handle {

            Some(ref handle) => {
                info!("[Server] [{}] [Voice] Resetting existing handle.", self);
                handle.send(QueueEntry::Reset).ok();
            }

            None => {
                info!("[Server] [{}] [Voice] Creating new handle.", self);

                let mut listener = Listener::new(
                    self.voice_queue.clone(),
                    self.voice_listener_count.clone()
                );

                self.voice_listener_handle = listener.take_handle();

                voice_connection.set_receiver(Box::new(listener));
                voice_connection.play(
                    Box::new(Mixer::new(self.voice_queue.clone()))
                );

            }

        }

        self.last_voice_channel = Some(channel_id);

    }

}


// Commands -------------------------------------------------------------------
impl Server {

    pub fn reload_configuration(&mut self) {
        self.effect_manager.load_effects();
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

    pub fn add_voice_state(
        &mut self,
        channel_id: ChannelId,
        voice: &VoiceState,
        mut user: User
    ) {
        if user.is_bot {
            info!("[Server] [{}] [{}] [Voice] Ignored state for bot.", self, user);

        } else {
            info!("[Server] [{}] [{}] [Voice] State added.", self, user);
            user.deaf = voice.deaf || voice.self_deaf;
            user.mute = voice.mute || voice.self_mute;
            self.voice_states.push((channel_id, user));
        }
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

