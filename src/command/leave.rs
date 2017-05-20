// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions, ServerActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![
            ServerActions::LeaveVoice::new(command.message),
            MessageActions::Send::single_public(
                &command.message,
                format!(
                    "{} has requested me to leave the voice channel.",
                    command.member.nickname
                )
            )
        ]
    }

    fn help(&self) -> &str {
        "Force the the bot to leave its current voice channel."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!leave`".to_string()
        )
    }

}

