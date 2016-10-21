// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, AliasActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![AliasActions::List::new(command.message)]
    }

}

