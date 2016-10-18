// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::action::{ActionGroup, BanActions};
use ::command::{Command, CommandHandler};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        _: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if !command.message.has_unique_server() {
            self.requires_unique_server(command)

        } else if !member.is_admin {
            self.requires_admin(command)

        } else {
            self.delete_and_send(
                command.message,
                BanActions::List::new(command.message)
            )
        }
    }

}

