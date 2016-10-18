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
        if !command.message.has_unique_server() {
            self.requires_unique_server(command)

        } else if !member.is_admin {
            self.requires_admin(command)

        } else if command.arguments.len() != 1 {
            self.delete_and_send(command.message, MessageActions::Send::public(
                &command.message,
                "Usage: `!delete <effect_name>`".to_string()
            ))

        } else if let Some(effect) = server.get_effect(&command.arguments[0]) {
            vec![EffectActions::Delete::new(
                command.message,
                effect
            )]

        } else {
            self.delete_and_send(command.message, MessageActions::Send::public(
                &command.message,
                format!(
                    "Sound effect `{}` does not exist and thus cannot be deleted.",
                    command.arguments[0]
                )
            ))
        }
    }

}

