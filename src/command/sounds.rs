// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, if command.arguments.is_empty() {
            EffectActions::List::all(command.message)

        } else {
            EffectActions::List::matching(command.message, command.arguments)
        })
    }

}

