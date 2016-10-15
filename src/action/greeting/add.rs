// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message,
    nickname: String,
    effect_name: String
}

impl ActionImpl {
    pub fn new(message: Message, nickname: String, effect_name: String) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            nickname: nickname,
            effect_name: effect_name
        })
    }
}

impl Action for ActionImpl {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            server.add_greeting(&self.nickname, &self.effect_name);
            vec![MessageActions::Send::private(&self.message, format!(
                "Greeting for `{}` has been set to `{}` on {}.",
                self.nickname, self.effect_name, server.name
            ))]

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [AddGreeting] {} {}", self.nickname, self.effect_name)
    }
}

