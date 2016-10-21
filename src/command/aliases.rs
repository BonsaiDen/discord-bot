// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, AliasActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        self.delete_and_send(command.message, AliasActions::List::new(
            command.message
        ))
    }

}

