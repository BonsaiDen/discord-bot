// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    UserId,
    ChannelId,
    ChannelType,
    ServerId,
    Channel as DiscordChannel,
    Permissions, PermissionOverwrite, PermissionOverwriteType
};


// Internal Dependencies ------------------------------------------------------
use ::core::member::Member;


// Channel Abstraction --------------------------------------------------------
#[derive(Debug)]
pub struct Channel {
    pub id: ChannelId,
    pub server_id: Option<ServerId>,
    pub name: String,
    bitrate: Option<u64>,
    is_voice: bool,
    voice_users: Vec<UserId>,
    permissions: Vec<PermissionOverwrite>
}


// Public Interface -----------------------------------------------------------
impl Channel {

    pub fn from_discord_channel(channel: DiscordChannel) -> Channel {
        match channel {
            DiscordChannel::Group(ref channel) => {
                Channel {
                    id: channel.channel_id,
                    server_id: None,
                    name: channel.name.clone().unwrap_or_else(|| {
                        "".to_string()

                    }).to_string(),
                    bitrate: None,
                    voice_users: Vec::new(),
                    permissions: Vec::new(),
                    is_voice: false
                }
            },
            DiscordChannel::Private(ref channel) => {
                Channel {
                    id: channel.id,
                    server_id: None,
                    name: "".to_string(),
                    bitrate: None,
                    voice_users: Vec::new(),
                    permissions: Vec::new(),
                    is_voice: false
                }
            },
            DiscordChannel::Public(ref channel) => {
                Channel {
                    id: channel.id,
                    server_id: Some(channel.server_id),
                    name: channel.name.to_string(),
                    bitrate: channel.bitrate,
                    voice_users: Vec::new(),
                    permissions: channel.permission_overwrites.clone(),
                    is_voice: channel.kind == ChannelType::Voice
                }
            }
        }
    }

    pub fn update(&mut self, channel: Channel) {
        self.name = channel.name;
        self.permissions = channel.permissions;
        self.bitrate = channel.bitrate;
    }

    pub fn bitrate(&self) -> u64 {
        (self.bitrate.unwrap_or(8000) as f64 / 1000.0).round() as u64
    }

    pub fn add_voice_member(&mut self, member_id: &UserId) {
        if !self.voice_users.contains(member_id) {
            self.voice_users.push(*member_id);
        }
    }

    pub fn remove_voice_member(&mut self, member_id: &UserId) {
        self.voice_users.retain(|id| id != member_id);
    }

    pub fn is_empty_voice(&self) -> bool {
        self.voice_users.is_empty()
    }

    pub fn voice_users(&self) -> &Vec<UserId> {
        &self.voice_users
    }

    pub fn get_member_permissions(&self, member: &Member) -> Permissions {

        let mut allowed_perms = Permissions::empty();
        let mut denied_perms = Permissions::empty();

        for overwrite in &self.permissions {
            match overwrite.kind {

                PermissionOverwriteType::Role(id) => if member.roles.contains(&id) {
                    allowed_perms.insert(overwrite.allow);
                    denied_perms.remove(overwrite.deny);
                },

                PermissionOverwriteType::Member(id) if id == member.id => {
                    allowed_perms.insert(overwrite.allow);
                    denied_perms.remove(overwrite.deny);
                },

                _ => {}

            }
        }

        allowed_perms - denied_perms

    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_voice {
            write!(
                f,
                "[Voice Channel {} {} speaker(s) {} kbps]",
                self.name,
                self.voice_users.len(),
                self.bitrate.unwrap_or(0) as f64 / 1000.0
            )

        } else {
            write!(f, "[Text Channel {}]", self.name)
        }
    }
}

