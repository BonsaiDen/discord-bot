// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, AddAlias, RemoveAlias, SendPrivateMessage};


// Command Implementation -----------------------------------------------------
pub struct AliasCommand;

impl AliasCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, SendPrivateMessage::new(
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
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "An alias named `{}` already exists on {}.",
                    alias, server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, AddAlias::new(
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
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "An alias named `{}` does not exist on {}.",
                    alias, server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, RemoveAlias::new(
                command.message,
                alias.to_string()
            ))
        }
    }

}

impl CommandImplementation for AliasCommand {

    fn run(
        &self,
        command: Command,
        server: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if command.message.origin == MessageOrigin::DirectMessage {
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

