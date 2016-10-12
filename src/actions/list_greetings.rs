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
pub struct ListGreetings {
    message: Message
}

impl ListGreetings {
    pub fn new(message: Message) -> Box<ListGreetings> {
        Box::new(ListGreetings {
            message: message
        })
    }
}

impl Action for ListGreetings {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let mut aliases = server.list_greetings();
            aliases.sort();

            let aliases: Vec<String> = aliases.into_iter().map(|(nickname, effect)| {
                format!("- `{}` -> `{}`", nickname, effect)

            }).collect();

            if aliases.is_empty() {
                vec![SendMessage::private(
                    &self.message,
                    "No user greetings found on the current server.".to_string()
                )]

            } else {
                list_lines("User Greetings", aliases, 25).into_iter().map(|text| {
                    SendMessage::private(&self.message, text) as Box<Action>

                }).collect()
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListGreetings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListGreetings]")
    }
}

