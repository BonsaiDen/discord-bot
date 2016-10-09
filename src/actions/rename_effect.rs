// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::SendPublicMessage;
use ::actions::{Action, ActionGroup};


// Effect Rename Action -------------------------------------------------------
pub struct RenameEffect {
    message: Message,
    effect: Effect,
    name: String
}

impl RenameEffect {
    pub fn new(message: Message, effect: Effect, name: String) -> Box<RenameEffect> {
        Box::new(RenameEffect {
            message: message,
            effect: effect,
            name: name
        })
    }
}

impl Action for RenameEffect {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {
            info!("{} Renaming...", self);
            if let Err(err) = server.rename_effect(&self.effect, &self.name) {
                warn!("{} Renaming failed: {}", self, err);
                vec![SendPublicMessage::new(
                    &self.message,
                    format!(
                        "Failed to rename sound effect `{}` to `{}`.",
                        self.effect.name, self.name
                    )
                )]

            } else {
                warn!("{} Renaming successful.", self);
                vec![SendPublicMessage::new(
                    &self.message,
                    format!(
                        "Sound effect `{}` was renamed to `{}`.",
                        self.effect.name, self.name
                    )
                )]
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for RenameEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [RenameEffect] {} to \"{}\"", self.effect, self.name)
    }
}

