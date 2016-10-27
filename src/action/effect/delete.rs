// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message,
    effect: Effect
}

impl Action {
    pub fn new(message: Message, effect: &Effect) -> Box<Action> {
        Box::new(Action {
            message: message,
            effect: effect.clone()
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if let Err(err) = server.delete_effect(&self.effect) {
                warn!("{} Deletion failed: {}", self, err);
                MessageActions::Send::public(
                    &self.message,
                    format!(
                        "Failed to delete sound effect `{}`.",
                        self.effect.name
                    )
                )

            } else {
                MessageActions::Send::public(
                    &self.message,
                    format!(
                        "Sound effect `{}` was deleted.",
                        self.effect.name
                    )
                )
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [DeleteEffect] {}", self.effect)
    }
}

