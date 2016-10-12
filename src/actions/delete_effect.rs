// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;
use ::actions::SendMessage;
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct DeleteEffect {
    message: Message,
    effect: Effect
}

impl DeleteEffect {
    pub fn new(message: Message, effect: Effect) -> Box<DeleteEffect> {
        Box::new(DeleteEffect {
            message: message,
            effect: effect
        })
    }
}

impl Action for DeleteEffect {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {



            if let Err(err) = server.delete_effect(&self.effect) {
                warn!("{} Deletion failed: {}", self, err);
                vec![SendMessage::public(
                    &self.message,
                    format!(
                        "Failed to delete sound effect `{}`.",
                        self.effect.name
                    )
                )]

            } else {
                vec![SendMessage::public(
                    &self.message,
                    format!(
                        "Sound effect `{}` was deleted.",
                        self.effect.name
                    )
                )]
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for DeleteEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [DeleteEffect] {}", self.effect)
    }
}

