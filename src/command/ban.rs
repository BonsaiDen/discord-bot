// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, BanActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    require_min_arguments!(2);

    fn run(&self, command: Command) -> ActionGroup {
        match command.arguments[0].as_str() {
            "add" => self.add(&command, &command.arguments[1]),
            "remove" => self.remove(&command, &command.arguments[1]),
            _ => self.usage(command)
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, MessageActions::Send::private(
            &command.message,
            "Usage: `!ban add <user#ident>` or `!ban remove <user#ident>`".to_string()
        ))
    }

}

impl Handler {

    fn add(&self, command: &Command, nickname: &str) -> ActionGroup {
        if !command.server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, command.server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, BanActions::Add::new(
                command.message,
                nickname.to_string()
            ))
        }
    }

    fn remove(&self, command: &Command, nickname: &str) -> ActionGroup {
        if !command.server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, command.server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, BanActions::Remove::new(
                command.message,
                nickname.to_string()
            ))
        }
    }

}

