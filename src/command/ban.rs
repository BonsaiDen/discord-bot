// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, SendPrivateMessage, AddBan, RemoveBan};


// Command Implementation -----------------------------------------------------
pub struct BanCommand;

impl BanCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        vec![
            DeleteMessage::new(command.message),
            SendPrivateMessage::new(
                &command.message,
                "Usage:
                `!ban add <user#ident>` or
                `!ban remove <user#ident>`".to_string()
            )
        ]
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        nickname: &str,

    ) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            vec![
                DeleteMessage::new(command.message),
                SendPrivateMessage::new(
                    &command.message,
                    format!("The user `{}` was not found on the current server.", nickname)
                )
            ]

        } else {
            vec![
                DeleteMessage::new(command.message),
                AddBan::new(command.message, nickname.to_string())
            ]
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

        } else {
            vec![
                DeleteMessage::new(command.message),
                RemoveBan::new(command.message, nickname.to_string())
            ]
        }
    }

}

impl CommandImplementation for BanCommand {

    fn run(
        &self,
        command: Command,
        server: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if command.message.origin == MessageOrigin::DirectMessage {
            self.requires_unique_server(command)

        } else if !member.is_admin {
            self.requires_admin(command)

        } else if command.arguments.len() < 3 {
            self.usage(command)

        } else {
            match command.arguments[0].as_str() {
                "add" => self.add(
                    server,
                    &command,
                    &command.arguments[2],
                ),
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

