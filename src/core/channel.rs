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
    pub is_voice: bool,
    voice_members: Vec<UserId>,
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
                    voice_members: Vec::new(),
                    permissions: Vec::new(),
                    is_voice: false
                }
            },
            DiscordChannel::Private(ref channel) => {
                Channel {
                    id: channel.id,
                    server_id: None,
                    name: "".to_string(),
                    voice_members: Vec::new(),
                    permissions: Vec::new(),
                    is_voice: false
                }
            },
            DiscordChannel::Public(ref channel) => {
                Channel {
                    id: channel.id,
                    server_id: Some(channel.server_id),
                    name: channel.name.to_string(),
                    voice_members: Vec::new(),
                    permissions: channel.permission_overwrites.clone(),
                    is_voice: channel.kind == ChannelType::Voice
                }
            }
        }
    }

    pub fn update(&mut self, channel: Channel) {
        self.name = channel.name;
        self.permissions = channel.permissions;
    }

    pub fn add_voice_member(&mut self, member_id: &UserId) {
        if !self.voice_members.contains(member_id) {
            self.voice_members.push(*member_id);
        }
    }

    pub fn remove_voice_member(&mut self, member_id: &UserId) {
        self.voice_members.retain(|id| id != member_id);
    }

    pub fn is_empty_voice(&self) -> bool {
        self.voice_members.is_empty()
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
                "[Voice Channel {} {} voice(s)]",
                self.name, self.voice_members.len()
            )

        } else {
            write!(f, "[Text Channel {}]", self.name)
        }
    }
}

