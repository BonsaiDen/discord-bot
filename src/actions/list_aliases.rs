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
pub struct ListAliases {
    message: Message
}

impl ListAliases {
    pub fn new(message: Message) -> Box<ListAliases> {
        Box::new(ListAliases {
            message: message
        })
    }
}

impl Action for ListAliases {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let mut aliases = server.list_aliases();
            aliases.sort();

            let aliases: Vec<String> = aliases.into_iter().map(|(name, effects)| {
                format!("- `{}` -> `{}`", name, effects.join("`, `"))

            }).collect();

            if aliases.is_empty() {
                vec![SendMessage::private(
                    &self.message,
                    "No effect aliases found on the current server.".to_string()
                )]

            } else {
                list_lines("Effect Aliases", aliases, 25).into_iter().map(|text| {
                    SendMessage::private(&self.message, text) as Box<Action>

                }).collect()
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListAliases {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListAliases]")
    }
}

