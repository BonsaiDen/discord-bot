// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, BanActions, MessageActions};


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
                self.add(&command, &command.arguments[1])
            },
            "remove" => if command.arguments.len() < 2 {
                self.usage(command)

            } else {
                self.remove(&command, &command.arguments[1])
            },
            "list" => vec![BanActions::List::new(command.message)],
            _ => self.usage(command)
        }
    }

    fn help(&self) -> &str {
        "List, add or remove banned users."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!ban add <user#ident>` or `!ban remove <user#ident>` or `!ban list`".to_string()
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
            vec![BanActions::Add::new(command.message, nickname.to_string())]
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
            vec![BanActions::Remove::new(command.message, nickname.to_string())]
        }
    }

}

