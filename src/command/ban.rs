// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::command::{Command, CommandHandler};
use ::core::{Member, MessageOrigin, Server};
use ::action::{ActionGroup, BanActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandImpl {

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, MessageActions::Send::private(
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
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!("The user `{}` is not a member of {}.", nickname, server.name)
            ))

        } else {
            self.delete_and_send(command.message, BanActions::Add::new(
                command.message,
                nickname.to_string()
            ))
        }
    }

    fn remove(&self, server: &Server, command: &Command, nickname: &str) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!("The user `{}` is not a member of {}.", nickname, server.name)
            ))

        } else {
            self.delete_and_send(command.message, BanActions::Remove::new(
                command.message,
                nickname.to_string()
            ))
        }
    }

}

impl CommandHandler for CommandImpl {

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

