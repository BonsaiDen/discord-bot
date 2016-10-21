// Internal Dependencies ------------------------------------------------------
use ::action::{ActionGroup, BanActions};
use ::command::{Command, CommandHandler};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();

    fn run(&self, command: Command) -> ActionGroup {
        self.delete_and_send(
            command.message,
            BanActions::List::new(command.message)
        )
    }

}

