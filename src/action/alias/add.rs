// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message,
    name: String,
    effect_names: Vec<String>
}

impl Action {
    pub fn new(message: Message, name: String, effect_names: Vec<String>) -> Box<Action> {
        Box::new(Action {
            message: message,
            name: name,
            effect_names: effect_names
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.add_alias(&self.name, &self.effect_names);
            MessageActions::Send::private(&self.message, format!(
                "`{}` is now an alias for `{}` on {}.",
                self.name, self.effect_names.join("`, `"), server.name
            ))

        } else {
            vec![]
        }

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [AddAlias] {} for \"{}\"",
            self.name,
            self.effect_names.join("\", \"")
        )
    }
}

