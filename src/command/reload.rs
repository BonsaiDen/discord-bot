// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions, ServerActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![
            ServerActions::Reload::new(command.message),
            MessageActions::Delete::new(command.message),
            MessageActions::Send::public(
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

