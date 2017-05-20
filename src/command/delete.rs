// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    require_exact_arguments!(1);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        if let Some(effect) = command.server.get_effect(&command.arguments[0]) {
            vec![EffectActions::Delete::new(command.message, effect)]

        } else {
            MessageActions::Send::public(
                &command.message,
                format!(
                    "Sound effect `{}` does not exist and thus cannot be deleted.",
                    command.arguments[0]
                )
            )
        }
    }

    fn help(&self) -> &str {
        "Fully delete existing sound effects."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::public(
            &command.message,
            "Usage: `!delete <effect_name>`".to_string()
        )
    }

}

