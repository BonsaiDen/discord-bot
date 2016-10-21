// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, GreetingActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_min_arguments!(2);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        match command.arguments[0].as_str() {
            "add" => if command.arguments.len() < 3 {
                self.usage(command)

            } else {
                self.add(
                    &command,
                    &command.arguments[1],
                    &command.arguments[2]
                )
            },
            "remove" => self.remove(&command, &command.arguments[1]),
            _ => self.usage(command)
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!greeting add <user#ident> <effect_name>` or `!greeting remove <user#ident>`".to_string()
        )
    }

}

impl Handler {

    fn add(
        &self,
        command: &Command,
        nickname: &str,
        effect_name: &str

    ) -> ActionGroup {
        if !command.server.has_member_with_nickname(nickname) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, command.server.name
                )
            )

        } else if command.server.has_greeting(nickname) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "A greeting for the user `{}` already exists on {}, please remove it first.",
                    nickname, command.server.name
                )
            )

        } else if command.server.has_matching_effects(effect_name, command.config) {
            vec![GreetingActions::Add::new(
                command.message,
                nickname.to_string(),
                effect_name.to_string()
            )]

        } else {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "Cannot add a greeting when there are no effects matching `{}` on {}.",
                    nickname, command.server.name
                )
            )
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

        } else if command.server.has_greeting(nickname) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "A greeting for the user `{}` does not exist on {}.",
                    nickname, command.server.name
                )
            )

        } else {
            vec![GreetingActions::Remove::new(command.message, nickname.to_string())]
        }
    }

}

