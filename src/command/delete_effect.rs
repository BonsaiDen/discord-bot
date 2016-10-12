// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteEffect, SendMessage};


// Command Implementation -----------------------------------------------------
pub struct DeleteEffectCommand;

impl CommandImplementation for DeleteEffectCommand {

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

        } else if command.arguments.len() != 1 {
            self.delete_and_send(command.message, SendMessage::public(
                &command.message,
                "Usage: `!delete <effect_name>`".to_string()
            ))

        } else if !server.has_effect(&command.arguments[0]) {
            self.delete_and_send(command.message, SendMessage::public(
                &command.message,
                format!(
                    "Sound effect `{}` does not exist and thus cannot be deleted.",
                    command.arguments[0]
                )
            ))

        } else {
            vec![DeleteEffect::new(
                command.message,
                server.get_effect(&command.arguments[0]).unwrap()
            )]
        }
    }

}

