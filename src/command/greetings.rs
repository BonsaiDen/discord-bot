// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, GreetingActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        _: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if !command.message.origin.is_unique() {
            self.requires_unique_server(command)

        } else {
            self.delete_and_send(
                command.message,
                GreetingActions::List::new(command.message)
            )
        }
    }

}

