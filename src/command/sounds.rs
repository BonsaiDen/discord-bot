// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        if command.arguments.is_empty() {
            vec![EffectActions::List::all(command.message)]

        } else {
            vec![EffectActions::List::matching(command.message, command.arguments)]
        }
    }

}

