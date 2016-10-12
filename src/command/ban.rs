// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, AddBan, RemoveBan, SendMessage};


// Command Implementation -----------------------------------------------------
pub struct BanCommand;

impl BanCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, SendMessage::private(
            &command.message,
            "Usage: `!ban add <user#ident>` or `!ban remove <user#ident>`".to_string()
        ))
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        nickname: &str,

    ) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, SendMessage::private(
                &command.message,
                format!("The user `{}` is not a member of {}.", nickname, server.name)
            ))

        } else {
            self.delete_and_send(command.message, AddBan::new(command.message, nickname.to_string()))
        }
    }

    fn remove(&self, server: &Server, command: &Command, nickname: &str) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, SendMessage::private(
                &command.message,
                format!("The user `{}` is not a member of {}.", nickname, server.name)
            ))

        } else {
            self.delete_and_send(command.message, RemoveBan::new(command.message, nickname.to_string()))
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

        } else if command.arguments.len() < 2 {
            self.usage(command)

        } else {
            match command.arguments[0].as_str() {
                "add" => self.add(
                    server,
                    &command,
                    &command.arguments[1],
                ),
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

