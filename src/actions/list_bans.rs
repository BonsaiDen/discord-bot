// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::actions::SendMessage;
use ::core::message::Message;
use ::core::event::EventQueue;
use ::text_util::list_lines;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct ListBans {
    message: Message
}

impl ListBans {
    pub fn new(message: Message) -> Box<ListBans> {
        Box::new(ListBans {
            message: message
        })
    }
}

impl Action for ListBans {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let mut bans = server.list_bans();
            bans.sort();

            if bans.is_empty() {
                vec![SendMessage::private(
                    &self.message,
                    format!("There are currently no banned users on {}.", server.name)
                )]

            } else {
                let title = format!("Banned Users on {}", server.name);
                list_lines(&title, bans, 25).into_iter().map(|text| {
                    SendMessage::private(&self.message, text) as Box<Action>

                }).collect()
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListBans {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListBans]")
    }
}

