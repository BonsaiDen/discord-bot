// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::actions::SendMessage;
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct AddGreeting {
    message: Message,
    nickname: String,
    effect_name: String
}

impl AddGreeting {
    pub fn new(message: Message, nickname: String, effect_name: String) -> Box<AddGreeting> {
        Box::new(AddGreeting {
            message: message,
            nickname: nickname,
            effect_name: effect_name
        })
    }
}

impl Action for AddGreeting {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.add_greeting(&self.nickname, &self.effect_name);
            vec![SendMessage::private(&self.message, format!(
                "Greeting for `{}` has been set to `{}` on {}.",
                self.nickname, self.effect_name, server.name
            ))]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for AddGreeting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [AddGreeting] {} {}", self.nickname, self.effect_name)
    }
}

