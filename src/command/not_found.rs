// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, SendPrivateMessage};


// Command Implementation -----------------------------------------------------
pub struct NotFoundCommand;

impl CommandImplementation for NotFoundCommand {

    fn run(
        &self,
        command: Command,
        _: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        vec![SendPrivateMessage::new(
            &command.message,
            format!(
                "The command `{}` does not exist, please type
                `!help` for a list of all available commands.",
                command.name
            )
        )]
    }

}
