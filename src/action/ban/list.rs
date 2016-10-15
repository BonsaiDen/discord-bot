// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::text_util::list_lines;
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message
}

impl ActionImpl {
    pub fn new(message: Message) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message
        })
    }
}

impl Action for ActionImpl {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let mut bans = server.list_bans();
            bans.sort();

            if bans.is_empty() {
                vec![MessageActions::Send::private(
                    &self.message,
                    format!("There are currently no banned users on {}.", server.name)
                )]

            } else {
                let title = format!("Banned Users on {}", server.name);
                list_lines(&title, bans, 25).into_iter().map(|text| {
                    MessageActions::Send::private(&self.message, text) as Box<Action>

                }).collect()
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListBans]")
    }
}

