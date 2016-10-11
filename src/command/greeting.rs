// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, SendPrivateMessage};


// Command Implementation -----------------------------------------------------
pub struct GreetingCommand;

impl GreetingCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        vec![
            DeleteMessage::new(command.message),
            SendPrivateMessage::new(
                &command.message,
                "Usage:
                `!greeting add <user#ident> <effect_name>` or
                `!greeting remove <user#ident>`".to_string()
            )
        ]
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        nickname: &str,
        _: &str

    ) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    format!("The user `{}` was not found on the current server.", nickname)
                )
            ]

        } else if server.has_greeting(nickname) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    format!("A greeting for the user `{}` already exists on the current server.", nickname)
                )
            ]

        } else {
            // TODO check if the effects exist
            // TODO add alias
            vec![]
        }
    }

    fn remove(&self, server: &Server, command: &Command, nickname: &str) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    format!("The user `{}` was not found on the current server.", nickname)
                )
            ]

        } else if server.has_greeting(nickname) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    format!("A greeting for the user `{}` does not exist on the current server.", nickname)
                )
            ]

        } else {
            vec![
                DeleteMessage::new(command.message),
                // TODO remove RemoveGreeting::new(command.message, nickname)
                //SendPrivateMessage::new(
                //    &command.message,
                //    "Alias `{}` was removed from the current server.".to_string()
                //)
            ]
        }
    }

}

impl CommandImplementation for GreetingCommand {

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
                        &command.arguments[3]
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

