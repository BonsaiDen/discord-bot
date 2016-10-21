// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![
            EffectActions::Silence::new(command.message),
            MessageActions::Delete::new(command.message),
            MessageActions::Send::public(
                &command.message,
                format!(
                    "{} has requested me to stay quiet.",
                    command.member.nickname
                )
            )
        ]
    }

}


