// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::atomic::Ordering;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId, VoiceState};
use discord::model::permissions::{VOICE_CONNECT, VOICE_SPEAK};


// External Dependencies ------------------------------------------------------
use chrono;
use edit_distance::edit_distance;


// Internal Dependencies ------------------------------------------------------
mod config;
use self::config::{ConfigHandler, ConfigRef};

use super::{Handle, User, Effect, EffectManager};
use super::voice::{
    Listener, ListenerCount, EmptyListenerCount,
    Mixer, Greeting, MixerCount, EmptyMixerCount,
    Queue, QueueHandle, QueueEntry, EmptyQueue
};


// Statics --------------------------------------------------------------------
static MILLIS_EFFECT_JOIN_DELAY: usize = 300;
static SECS_USER_GREETING_DELAY: i64 = 60;


// Enums ----------------------------------------------------------------------
#[derive(PartialEq)]
enum VoiceJoinResult {
    Joining,
    AlreadyPresent,
    Ignored
}


// Server Abstraction ---------------------------------------------------------
pub struct Server {

    // General
    id: ServerId,
    name: String,
    channel_count: usize,
    member_count: usize,
    config: ConfigHandler,

    // Users
    users_admins: Vec<String>,
    users_uploaders: Vec<String>,

    // Voice
    last_voice_channel: Option<ChannelId>,
    voice_mixer_count: MixerCount,
    voice_listener_handle: Option<QueueHandle>,
    voice_listener_count: ListenerCount,
    voice_states: Vec<(ChannelId, User)>,
    voice_greetings: HashMap<String, Greeting>,
    voice_queue: Queue,

    // Effects
    effect_manager: EffectManager

}

impl Server {

    pub fn new(
        id: ServerId,
        effects_directory: PathBuf,
        config_directory: PathBuf

    ) -> Server {

        Server {

            // General
            id: id,
            name: "???".to_string(),
            channel_count: 0,
            member_count: 0,
            config: ConfigHandler::new(id, config_directory),

            // Admins
            users_admins: Vec::new(),
            users_uploaders: Vec::new(),

            // Voice
            last_voice_channel: None,
            voice_mixer_count: EmptyMixerCount::create(),
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

// Sounds Effect --------------------------------------------------------------
impl Server {

    pub fn play_effects(
        &mut self,
        handle: &mut Handle,
        channel_id: ChannelId,
        effects: Vec<Effect>,
        immediate: bool,
        mut delay: usize
    ) {

        let state = self.join_voice_channel(handle, Some(channel_id));
        if state != VoiceJoinResult::Ignored {

            if state == VoiceJoinResult::Joining {
                delay += MILLIS_EFFECT_JOIN_DELAY;
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

    pub fn list_effects(&self) -> Vec<&str> {
        self.effect_manager.list_effects()
    }

    pub fn get_aliases(&self) -> &HashMap<String, Vec<String>> {
        self.effect_manager.get_aliases()
    }

    pub fn list_aliases(&self) -> Vec<String> {
        self.effect_manager.get_aliases().iter().map(|(name, effects)| {
            let names = effects.iter().map(|e| {
                e.as_str()

            }).collect::<Vec<&str>>().join("`, `");
            format!("{} **=>** `{}`", name, names)

        }).collect()
    }

    pub fn list_greetings(&self) -> Vec<String> {
        self.voice_greetings.values().filter(|greeting| {
            greeting.permanent

        }).map(|greeting| {
            format!("{} **=>** `{}`", greeting.nickname, greeting.effect)

        }).collect()
    }

    pub fn get_effect_suggestions(
        &self,
        name: &str,
        max_distance: u32,
        count: usize

    ) -> Vec<&str> {

        let mut suggestions: Vec<(u32, &str)> = self.list_effects().iter().map(|effect| {
            if effect.contains(name) {
                (max_distance / 2, *effect)

            } else {
                (edit_distance(name, effect).abs() as u32, *effect)
            }

        }).filter(|&(l, _)| l < max_distance).collect();

        suggestions.sort();
        suggestions.iter().map(|&(_, s)| s).take(count).collect()

    }

    pub fn download_effect(&mut self, effect: &str, url: &str) -> Result<(), String> {
        self.effect_manager.download_effect(effect, url)
    }

    pub fn delete_effect(&mut self, effect: &str) -> Result<(), String> {
        self.effect_manager.delete_effect(effect)
    }

}


// Voice Handling -------------------------------------------------------------
impl Server {

    pub fn initialize(&mut self, handle: &mut Handle) {
        info!("[Server] [{}] Initializing.", self);
        self.join_voice_channel(handle, None);
    }

    pub fn update_voice(
        &mut self,
        handle: &mut Handle,
        voice: VoiceState,
        user: User
    ) {

        if user.id == handle.user_id() {
            if let Some(_) = voice.channel_id {
                info!("[Server] [{}] [Voice] Joined channel.", self);

            } else if self.voice_listener_handle.is_some() {
                info!("[Server] [{}] [Voice] Left channel.", self);
                self.voice_listener_handle = None;

            } else {
                info!("[Server] [{}] [Voice] Ignored leave from previous connection.", self);
            }

        } else if let Some(channel_id) = voice.channel_id {
            if self.voice_states.iter().any(|&(_, ref u)| u.id == user.id) {
                if self.update_voice_state(channel_id, voice, &user) {
                    info!("[Server] [{}] [{}] [Voice] User switched channel.", self, user);
                    self.greet_user(handle, channel_id, &user);

                } else {
                    info!("[Server] [{}] [{}] [Voice] User state updated.", self, user);
                }

            } else {
                info!("[Server] [{}] [{}] [Voice] User joined channel.", self, user);
                self.greet_user(handle, channel_id, &user);
                self.add_voice_state(channel_id, &voice, user);
            }

        } else {
            info!("[Server] [{}] [{}] [Voice] User left channel.", self, user);
            self.voice_states.retain(|&(_, ref u)| u.id != user.id);
        }

        // Only do this as long as we are connected, otherwise we'll be leaking
        // voice threads
        self.update_voice_count(handle);

    }

    pub fn leave_voice_channel(&mut self, handle: &mut Handle) {
        handle.disconnect_server_voice(self.id);
        self.last_voice_channel = None;
    }

    fn update_voice_count(&mut self, handle: &mut Handle) {

        if let Some(channel_id) = self.last_voice_channel {

            // Update Active Listener Count
            let active_listener_count = self.voice_states
                .iter()
                .filter(|&&(ref id, ref user)| {
                    *id == channel_id && !user.mute && !user.deaf

                }).count();

            self.voice_listener_count.store(active_listener_count, Ordering::Relaxed);

            // Check channel user count and leave if empty
            let channel_user_count = self.voice_states
                .iter()
                .filter(|&&(ref id, _)| {
                    *id == channel_id

                }).count();

            if channel_user_count == 0 {
                info!("[Server] [{}] [Voice] Leaving empty channel.", self);
                handle.disconnect_server_voice(self.id)
            }

        }

    }

    fn join_voice_channel(&mut self, handle: &mut Handle, channel_id: Option<ChannelId>) -> VoiceJoinResult {

        if let Some(target_id) = channel_id.or(self.last_voice_channel) {

            let permissions = handle.get_channel_permissions(&target_id);
            let mixer_count = self.voice_mixer_count.load(Ordering::Relaxed);

            if !permissions.contains(VOICE_CONNECT | VOICE_SPEAK) {
                info!("[Server] [{}] [Voice] Ignoring channel with insufficient permissions.", self);
                VoiceJoinResult::Ignored

            } else if mixer_count == 0 {
                info!("[Server] [{}] [Voice] Joining channel.", self);
                self.init_voice_connection(handle, target_id);
                VoiceJoinResult::Joining

            } else if channel_id.is_none() {
                if mixer_count == 0 {
                    info!("[Server] [{}] [Voice] Re-joining channel.", self);
                    self.init_voice_connection(handle, target_id);
                    VoiceJoinResult::Joining

                } else {
                    info!("[Server] [{}] [Voice] Already in channel.", self);
                    VoiceJoinResult::AlreadyPresent
                }

            } else if channel_id == self.last_voice_channel {
                info!("[Server] [{}] [Voice] Already in channel.", self);
                VoiceJoinResult::AlreadyPresent

            } else {
                info!("[Server] [{}] [Voice] Switching channel.", self);
                self.init_voice_connection(handle, target_id);
                VoiceJoinResult::Joining
            }

        } else {
            VoiceJoinResult::Ignored
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
                    Box::new(Mixer::new(
                        self.voice_queue.clone(),
                        self.voice_mixer_count.clone()
                    ))
                );

            }

        }

        self.last_voice_channel = Some(channel_id);

    }

    fn greet_user(
        &mut self,
        handle: &mut Handle,
        channel_id: ChannelId,
        user: &User
    ) {

        // TODO ignore muted / deafed users?

        let mut user_greeting = None;
        if let Some(greeting) = self.get_user_greeting(user) {

            let now = chrono::Local::now().num_seconds_from_unix_epoch();
            let diff = now - greeting.last_played;
            if diff > SECS_USER_GREETING_DELAY {
                greeting.last_played = now;
                user_greeting = Some((user, vec![greeting.effect.to_string()]));
            }

        }

        if let Some((user, names)) = user_greeting {
            let effects = self.map_effects(&names);
            if !effects.is_empty() {
                info!(
                    "[Server] [{}] [{}] [Voice] Greeting with \"{}\".",
                    self, user, names.join("\", \"")
                );
                self.play_effects(handle, channel_id, effects, true, 300);
            }
        }

    }

    fn get_user_greeting(&mut self, user: &User) -> Option<&mut Greeting> {

        if !self.voice_greetings.contains_key(&user.nickname) {
            if let Some(default) = self.get_default_greeting() {
                self.voice_greetings.insert(
                    user.nickname.clone(),
                    Greeting::new(user.nickname.clone(), default, false)
                );
            }
        }

        self.voice_greetings.get_mut(&user.nickname)

    }

    fn get_default_greeting(&self) -> Option<String> {
        if let Some(ref default) = self.voice_greetings.get("default") {
            Some(default.effect.clone())

        } else {
            None
        }
    }

}


// Configuration --------------------------------------------------------------
impl Server {

    pub fn add_user_greeting(&mut self, nickname: &str, greeting: &str) {
        self.voice_greetings.insert(
            nickname.to_string(),
            Greeting::new(nickname.to_string(), greeting.to_string(), true)
        );
        self.store_config();
    }

    pub fn remove_user_greeting(&mut self, nickname: &str) {
        if let Some(_) = self.voice_greetings.remove(nickname) {
            self.store_config();
        }
    }

    pub fn add_effect_alias(&mut self, name: &str, effects: Vec<String>) {
        self.effect_manager.add_alias(name, effects);
        self.store_config();
    }

    pub fn remove_effect_alias(&mut self, name: &str) {
        if let Some(_) = self.effect_manager.remove_alias(name) {
            self.store_config();
        }
    }

    pub fn load_config(&mut self) {

        self.effect_manager.load_effects();

        match self.config.load() {
            Ok(config) => {
                self.effect_manager.set_aliases(config.aliases);
                self.voice_greetings = config.greetings;
                self.users_admins = config.admins;
                self.users_uploaders = config.uploaders;
                info!("[Server] [{}] [Config] [Load] Configuration loaded successfully.", self)
            }
            Err(err) => {
                warn!("[Server] [{}] [Config] [Load] Configuration could not be loaded: {}", self, err)
            }
        }

    }

    pub fn store_config(&self) {
        match self.config.store(self.get_config()) {
            Ok(_) => info!("[Server] [{}] [Config] [Store] Configuration stored successfully.", self),
            Err(err) => warn!("[Server] [{}] [Config] [Store] Configuration could not be stored: {}", self, err)
        }
    }

    fn get_config<'a>(&'a self) -> ConfigRef<'a> {
        ConfigRef {
            aliases: self.effect_manager.get_aliases(),
            greetings: &self.voice_greetings,
            uploaders: &self.users_uploaders,
            admins: &self.users_admins
        }
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

    pub fn is_admin_user(&self, user: &User) -> bool {
        self.users_admins.contains(&user.nickname)
    }

    pub fn is_upload_user(&self, user: &User) -> bool {
        self.users_uploaders.contains(&user.nickname)
    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

