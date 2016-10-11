// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::collections::HashMap;


// Discord Dependencies -------------------------------------------------------
use discord::model::{
    ChannelId, UserId, RoleId, ServerId,
    Member as DiscordMember,
    User as DiscordUser
};


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Member Abstraction ---------------------------------------------------------
#[derive(Debug, PartialEq)]
pub struct Member {
    pub id: UserId,
    pub server_id: ServerId,
    pub name: String,
    pub roles: Vec<RoleId>,
    pub nickname: String,
    pub is_bot: bool,
    pub is_active_bot: bool,
    pub is_admin: bool,
    pub is_uploader: bool,
    pub is_banned: bool,
    pub voice_channel_id: Option<ChannelId>,
    pub last_voice_leave: HashMap<ChannelId, u64>,
    pub mute: bool,
    pub deaf: bool
}


// Public Interface -----------------------------------------------------------
impl Member {

    pub fn from_discord_member(
        discord_member: DiscordMember,
        server_id: ServerId,
        bot_config: &BotConfig

    ) -> Member {
        let mut member = Member::from_discord_user(
            discord_member.user,
            server_id,
            bot_config
        );
        member.roles = discord_member.roles.clone();
        member
    }

    fn from_discord_user(
        user: DiscordUser,
        server_id: ServerId,
        bot_config: &BotConfig

    ) -> Member {
        let nickname = format!("{}#{}", user.name, user.discriminator);
        Member {
            id: user.id,
            server_id: server_id,
            name: user.name.to_string(),
            roles: Vec::new(),
            nickname: nickname.clone(),
            is_bot: user.bot,
            is_active_bot: nickname == bot_config.bot_nickname,
            is_admin: false,
            is_uploader: false,
            is_banned: false,
            voice_channel_id: None,
            last_voice_leave: HashMap::new(),
            mute: false,
            deaf: false
        }
    }

    pub fn should_be_greeted(
        &mut self,
        bot_config: &BotConfig

    ) -> bool {
        if self.voice_channel_id.is_some() {
            let last_leave = self.last_voice_leave
                                 .entry(self.voice_channel_id.unwrap())
                                 .or_insert(0);

            clock_ticks::precise_time_ms() - *last_leave
                > bot_config.greeting_separation_ms

        } else {
            false
        }
    }

    pub fn left_channel(&mut self, channel_id: &ChannelId) {
        self.last_voice_leave.insert(
            *channel_id,
            clock_ticks::precise_time_ms()
        );
    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let kind = match (self.is_bot, self.is_admin, self.is_uploader, self.is_banned) {
            (_, _, _, true) => "<Banned>",
            (true, _, _, false) => "<Bot>",
            (false, true, true, false) => "<Admin, Uploader>",
            (false, true, false, false) => "<Admin>",
            (false, false, true, false) => "<Uploader>",
            (false, false, false, false) => ""
        };

        if self.voice_channel_id.is_some() {
            write!(f, "[Member {}{} (Voice) Mute: {} Deaf: {}]", self.nickname, kind, self.mute, self.deaf)

        } else {
            write!(f, "[Member {}{} (Text)]", self.nickname, kind)
        }

    }
}

