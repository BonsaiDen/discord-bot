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

            let mut aliases = server.list_aliases();
            aliases.sort();

            let aliases: Vec<String> = aliases.into_iter().map(|(name, effects)| {
                format!("- `{}` -> `{}`", name, effects.join("`, `"))

            }).collect();

            if aliases.is_empty() {
                vec![MessageActions::Send::private(
                    &self.message,
                    format!("No effect aliases found on {}.", server.name)
                )]

            } else {
                list_lines("Effect Aliases", aliases, 25).into_iter().map(|text| {
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
        write!(f, "[Action] [ListAliases]")
    }
}

