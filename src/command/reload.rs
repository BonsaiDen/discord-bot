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
            ServerActions::Reload::new(command.message),
            MessageActions::Send::single_public(
                &command.message,
                format!(
                    "{} requested a configuration reload for {}.",
                    command.member.nickname,
                    command.server.name
                )
            )
        ]
    }

}

