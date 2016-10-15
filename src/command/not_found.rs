// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


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
        vec![MessageActions::Send::private(
            &command.message,
            format!(
                "The command `{}` does not exist, please type
                `!help` for a list of all available commands.",
                command.name
            )
        )]
    }

}

