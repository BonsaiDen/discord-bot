// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    require_exact_arguments!(2);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        if command.server.has_effect(&command.arguments[1]) {
            MessageActions::Send::public(
                &command.message,
                format!(
                    "A sound effect named `{}` already exist on {}.",
                    command.arguments[1],
                    command.server.name
                )
            )

        } else if let Some(effect) = command.server.get_effect(&command.arguments[0]) {
            vec![EffectActions::Rename::new(
                command.message,
                effect,
                command.arguments[1].clone()
            )]

        } else {
            MessageActions::Send::public(
                &command.message,
                format!(
                    "A sound effect named `{}` does not exist on {}.",
                    command.arguments[0],
                    command.server.name
                )
            )
        }
    }

    fn help(&self) -> &str {
        "Ranem existing sound effects."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::public(
            &command.message,
            "Usage: `!rename <old_effect_name> <new_effect_name>`".to_string()
        )
    }

}

