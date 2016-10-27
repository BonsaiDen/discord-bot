// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::text_util::list_lines;
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message
}

impl Action {
    pub fn new(message: Message) -> Box<Action> {
        Box::new(Action {
            message: message
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let mut aliases = server.list_greetings();
            aliases.sort();

            let aliases: Vec<String> = aliases.into_iter().map(|(nickname, effect)| {
                format!("- `{}` -> `{}`", nickname, effect)

            }).collect();

            if aliases.is_empty() {
                MessageActions::Send::private(
                    &self.message,
                    format!("No user greetings found on {}.", server.name)
                )

            } else {
                list_lines("User Greetings", aliases, 25).into_iter().map(|text| {
                    MessageActions::Send::single_private(&self.message, text) as Box<ActionHandler>

                }).collect()
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListGreetings]")
    }
}

