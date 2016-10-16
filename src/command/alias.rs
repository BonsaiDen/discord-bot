// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::command::{Command, CommandHandler};
use ::core::{Member, Server};
use ::action::{ActionGroup, AliasActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandImpl {

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, MessageActions::Send::private(
            &command.message,
            "Usage: `!alias add <alias_name> <effect_name>...` or `!alias remove <alias_name>`".to_string()
        ))
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        alias: &str,
        effect_names: &[String]

    ) -> ActionGroup {
        if server.has_alias(alias) {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!(
                    "An alias named `{}` already exists on {}.",
                    alias, server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, AliasActions::Add::new(
                command.message,
                alias.to_string(),
                effect_names.iter().map(|e| e.to_string()).collect()
            ))
        }
    }

    fn remove(
        &self,
        server: &Server,
        command: &Command,
        alias: &str

    ) -> ActionGroup {
        if server.has_alias(alias) {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!(
                    "An alias named `{}` does not exist on {}.",
                    alias, server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, AliasActions::Remove::new(
                command.message,
                alias.to_string()
            ))
        }
    }

}

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        server: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if !command.message.origin.is_unique() {
            self.requires_unique_server(command)

        } else if command.arguments.len() < 2 {
            self.usage(command)

        } else {
            match command.arguments[0].as_str() {
                "add" => if command.arguments.len() < 3 {
                    self.usage(command)

                } else {
                    self.add(
                        server,
                        &command,
                        &command.arguments[1],
                        &command.arguments[2..]
                    )
                },
                "remove" => self.remove(
                    server,
                    &command,
                    &command.arguments[1],
                ),
                _ => self.usage(command)
            }
        }

    }

}

