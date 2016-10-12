// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, ListGreetings};


// Command Implementation -----------------------------------------------------
pub struct GreetingsCommand;

impl CommandImplementation for GreetingsCommand {

    fn run(
        &self,
        command: Command,
        _: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if command.message.origin == MessageOrigin::DirectMessage {
            self.requires_unique_server(command)

        } else {
            self.delete_and_send(
                command.message,
                ListGreetings::new(command.message)
            )
        }
    }

}

