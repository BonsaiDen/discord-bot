// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, SendPrivateMessage, AddGreeting, RemoveGreeting};


// Command Implementation -----------------------------------------------------
pub struct GreetingCommand;

impl GreetingCommand {

    fn usage(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, SendPrivateMessage::new(
            &command.message,
            "Usage: `!greeting add <user#ident> <effect_name>` or `!greeting remove <user#ident>`".to_string()
        ))
    }

    fn add(
        &self,
        server: &Server,
        command: &Command,
        nickname: &str,
        effect_name: &str,
        config: &BotConfig

    ) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, server.name
                )
            ))

        } else if server.has_greeting(nickname) {
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "A greeting for the user `{}` already exists on {}, please remove it first.",
                    nickname, server.name
                )
            ))

        } else if server.has_matching_effects(effect_name, config) {
            self.delete_and_send(command.message, AddGreeting::new(
                command.message,
                nickname.to_string(),
                effect_name.to_string()
            ))

        } else {
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "Cannot add a greeting when there are no effects matching `{}` on {}.",
                    nickname, server.name
                )
            ))
        }
    }

    fn remove(
        &self,
        server: &Server,
        command: &Command,
        nickname: &str

    ) -> ActionGroup {
        if !server.has_member_with_nickname(nickname) {
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "The user `{}` is not a member of {}.",
                    nickname, server.name
                )
            ))

        } else if server.has_greeting(nickname) {
            self.delete_and_send(command.message, SendPrivateMessage::new(
                &command.message,
                format!(
                    "A greeting for the user `{}` does not exist on {}.",
                    nickname, server.name
                )
            ))

        } else {
            self.delete_and_send(command.message, RemoveGreeting::new(
                command.message,
                nickname.to_string()
            ))
        }
    }

}

impl CommandImplementation for GreetingCommand {

    fn run(
        &self,
        command: Command,
        server: &Server,
        _: &Member,
        config: &BotConfig

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
                        &command.arguments[2],
                        config
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

