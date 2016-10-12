// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::actions::{ActionGroup, ListEffects};
use ::command::{Command, CommandImplementation};


// Command Implementation -----------------------------------------------------
pub struct SoundsCommand;

impl CommandImplementation for SoundsCommand {

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
            self.delete_and_send(command.message, if command.arguments.is_empty() {
                ListEffects::all(command.message)

            } else {
                ListEffects::matching(command.message, command.arguments)
            })
        }
    }

}

