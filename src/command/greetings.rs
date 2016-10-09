// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, ListGreetings};


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
            vec![
                DeleteMessage::new(command.message),
                ListGreetings::new(command.message)
            ]
        }
    }

}

