// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, SendPrivateMessage};


// Command Implementation -----------------------------------------------------
pub struct AliasCommand;

impl AliasCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        vec![
            DeleteMessage::new(command.message),
            SendPrivateMessage::new(
                &command.message,
                "Usage:
                `!alias add <alias_name> <effect_name>...` or
                `!alias remove <alias_name>`".to_string()
            )
        ]
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        name: &str,
        _: &[String]

    ) -> ActionGroup {
        if server.has_alias(name) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    "Alias `{}` already exists on the current server.".to_string()
                )
            ]

        } else {
            // TODO check if the effects exist
            // TODO add alias
            vec![]
        }
    }

    fn remove(&self, server: &Server, command: &Command, name: &str) -> ActionGroup {
        if server.has_alias(name) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    "Alias `{}` does not exist on the current server.".to_string()
                )
            ]

        } else {
            vec![
                DeleteMessage::new(command.message),
                // TODO remove RemoveAlias::new(command.message, name)
                //SendPrivateMessage::new(
                //    &command.message,
                //    "Alias `{}` was removed from the current server.".to_string()
                //)
            ]
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

        } else if command.arguments.len() < 3 {
            self.usage(command)

        } else {
            match command.arguments[0].as_str() {
                "add" => if command.arguments.len() < 4 {
                    self.usage(command)

                } else {
                    self.add(
                        server,
                        &command,
                        &command.arguments[2],
                        &command.arguments[3..]
                    )
                },
                "remove" => self.remove(
                    server,
                    &command,
                    &command.arguments[2],
                ),
                _ => self.usage(command)
            }
        }

    }

}

