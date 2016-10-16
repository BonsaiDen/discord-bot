// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        server: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if !command.message.origin.is_unique() {
            self.requires_unique_server(command)

        } else if !member.is_admin {
            self.requires_admin(command)

        } else if command.arguments.len() != 1 {
            self.delete_and_send(command.message, MessageActions::Send::public(
                &command.message,
                "Usage: `!delete <effect_name>`".to_string()
            ))

        } else if !server.has_effect(&command.arguments[0]) {
            self.delete_and_send(command.message, MessageActions::Send::public(
                &command.message,
                format!(
                    "Sound effect `{}` does not exist and thus cannot be deleted.",
                    command.arguments[0]
                )
            ))

        } else {
            vec![EffectActions::Delete::new(
                command.message,
                server.get_effect(&command.arguments[0]).unwrap()
            )]
        }
    }

}

