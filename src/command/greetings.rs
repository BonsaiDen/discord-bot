// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, GreetingActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        self.delete_and_send(
            command.message,
            GreetingActions::List::new(command.message)
        )
    }

}

