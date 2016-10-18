// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, AliasActions};


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
        if !command.message.has_unique_server() {
            self.requires_unique_server(command)

        } else {
            self.delete_and_send(command.message, AliasActions::List::new(
                command.message
            ))
        }
    }

}

