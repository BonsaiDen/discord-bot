// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    fn run(&self, command: Command) -> ActionGroup {
        vec![MessageActions::Send::private(
            &command.message,
            format!(
                "The command `{}` does not exist, please type
                `!help` for a list of all available commands.",
                command.name
            )
        )]
    }

}

