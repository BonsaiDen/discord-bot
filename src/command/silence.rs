// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![
            EffectActions::Silence::new(command.message),
            MessageActions::Send::single_public(
                &command.message,
                format!(
                    "{} has requested me to stay quiet.",
                    command.member.nickname
                )
            )
        ]
    }

    fn help(&self) -> &str {
        "Stop all currently playing sound effects."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!silence`".to_string()
        )
    }

}

