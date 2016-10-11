// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// User Ban Action ------------------------------------------------------------
pub struct RemoveBan {
    message: Message,
    nickname: String
}

impl RemoveBan {
    pub fn new(message: Message, nickname: String) -> Box<RemoveBan> {
        Box::new(RemoveBan {
            message: message,
            nickname: nickname
        })
    }
}

impl Action for RemoveBan {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            if server.has_member_with_nickname(&self.nickname) {
                server.add_ban(&self.nickname);
                // TODO add ban
                vec![]

            } else {
                vec![]
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for RemoveBan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RemoveBan] {}", self.nickname)
    }
}

