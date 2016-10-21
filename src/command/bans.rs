// Internal Dependencies ------------------------------------------------------
use ::action::{ActionGroup, BanActions};
use ::command::{Command, CommandHandler};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![BanActions::List::new(command.message)]
    }

}

