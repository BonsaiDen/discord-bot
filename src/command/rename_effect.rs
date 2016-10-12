// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, RenameEffect, SendPublicMessage};


// Command Implementation -----------------------------------------------------
pub struct RenameEffectCommand;

impl CommandImplementation for RenameEffectCommand {

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

        } else if command.arguments.len() != 2 {
            self.delete_and_send(command.message, SendPublicMessage::new(
                &command.message,
                "Usage: `!rename <old_effect_name> <new_effect_name>`".to_string()
            ))

        } else if !server.has_effect(&command.arguments[0]) {
            self.delete_and_send(command.message, SendPublicMessage::new(
                &command.message,
                format!(
                    "A sound effect named `{}` does not exist on {}.",
                    command.arguments[0],
                    server.name
                )
            ))

        } else if server.has_effect(&command.arguments[1]) {
            self.delete_and_send(command.message, SendPublicMessage::new(
                &command.message,
                format!(
                    "A sound effect named `{}` already exist on {}.",
                    command.arguments[1],
                    server.name
                )
            ))

        } else {
            vec![RenameEffect::new(
                command.message,
                server.get_effect(&command.arguments[0]).unwrap(),
                command.arguments[1].clone()
            )]
        }
    }

}

