// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message,
    effect: Effect,
    name: String
}

impl ActionImpl {
    pub fn new(message: Message, effect: &Effect, name: String) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            effect: effect.clone(),
            name: name
        })
    }
}

impl Action for ActionImpl {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            if let Err(err) = server.rename_effect(&self.effect, &self.name) {
                warn!("{} Renaming failed: {}", self, err);
                MessageActions::Send::public(
                    &self.message,
                    format!(
                        "Failed to rename sound effect `{}` to `{}`.",
                        self.effect.name, self.name
                    )
                )

            } else {
                MessageActions::Send::public(
                    &self.message,
                    format!(
                        "Sound effect `{}` was renamed to `{}`.",
                        self.effect.name, self.name
                    )
                )
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RenameEffect] {} to \"{}\"", self.effect, self.name)
    }
}

