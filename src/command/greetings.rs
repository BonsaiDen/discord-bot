// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, GreetingActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![GreetingActions::List::new(command.message)]
    }

}

