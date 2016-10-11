// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// User Ban Action ------------------------------------------------------------
pub struct AddBan {
    message: Message,
    nickname: String
}

impl AddBan {
    pub fn new(message: Message, nickname: String) -> Box<AddBan> {
        Box::new(AddBan {
            message: message,
            nickname: nickname
        })
    }
}

impl Action for AddBan {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            if server.has_member_with_nickname(&self.nickname) {
                server.remove_ban(&self.nickname);
                // TODO remove ban
                vec![]

            } else {
                vec![]
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for AddBan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [AddBan] {}", self.nickname)
    }
}

