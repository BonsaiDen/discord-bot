// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, AliasActions, MessageActions};


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
                    &command.arguments[2..]
                )
            },
            "remove" => self.remove(
                &command,
                &command.arguments[1],
            ),
            _ => self.usage(command)
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!alias add <alias_name> <effect_name>...` or `!alias remove <alias_name>`".to_string()
        )
    }

}

impl Handler {

    fn add(
        &self,
        command: &Command,
        alias: &str,
        effect_names: &[String]

    ) -> ActionGroup {
        if command.server.has_alias(alias) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "An alias named `{}` already exists on {}.",
                    alias, command.server.name
                )
            )

        } else {
            vec![AliasActions::Add::new(
                command.message,
                alias.to_string(),
                effect_names.iter().map(|e| e.to_string()).collect()
            )]
        }
    }

    fn remove(&self, command: &Command, alias: &str) -> ActionGroup {
        if command.server.has_alias(alias) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "An alias named `{}` does not exist on {}.",
                    alias, command.server.name
                )
            )

        } else {
            vec![AliasActions::Remove::new(command.message, alias.to_string())]
        }
    }

}

