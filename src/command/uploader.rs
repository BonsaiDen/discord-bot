// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, UploaderActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    require_min_arguments!(1);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        match command.arguments[0].as_str() {
            "add" => if command.arguments.len() < 2 {
                self.usage(command)

            } else {
                self.add(
                    &command,
                    command.server.name_to_nickname(&command.arguments[1])
                )
            },
            "remove" => if command.arguments.len() < 2 {
                self.usage(command)

            } else {
                self.remove(
                    &command,
                    command.server.name_to_nickname(&command.arguments[1])
                )
            },
            "list" => vec![UploaderActions::List::new(command.message)],
            _ => self.usage(command)
        }
    }

    fn help(&self) -> &str {
        "List, add or remove users whitelisted for uploadind sound effects."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!uploader add <user#ident>` or `!uploader remove <user#ident>` or `!uploader list`".to_string()
        )
    }

}

impl Handler {

    fn add(&self, command: &Command, nickname: &str) -> ActionGroup {
        if !command.server.has_member_with_nickname(nickname) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, command.server.name
                )
            )

        } else {
            vec![UploaderActions::Add::new(command.message, nickname.to_string())]
        }
    }

    fn remove(&self, command: &Command, nickname: &str) -> ActionGroup {
        if !command.server.has_member_with_nickname(nickname) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, command.server.name
                )
            )

        } else {
            vec![UploaderActions::Remove::new(command.message, nickname.to_string())]
        }
    }

}

