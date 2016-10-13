// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::actions::SendMessage;
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct AddAlias {
    message: Message,
    name: String,
    effect_names: Vec<String>
}

impl AddAlias {
    pub fn new(message: Message, name: String, effect_names: Vec<String>) -> Box<AddAlias> {
        Box::new(AddAlias {
            message: message,
            name: name,
            effect_names: effect_names
        })
    }
}

impl Action for AddAlias {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.add_alias(&self.name, &self.effect_names);
            vec![SendMessage::private(&self.message, format!(
                "`{}` is now an alias for `{}` on {}.",
                self.name, self.effect_names.join("`, `"), server.name
            ))]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for AddAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [AddAlias] {} for \"{}\"",
            self.name,
            self.effect_names.join("\", \"")
        )
    }
}

