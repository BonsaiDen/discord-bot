// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{
    ActionGroup, DeleteMessage, ListAllEffects, ListPatternEffects
};


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
            vec![
                DeleteMessage::new(command.message),
                if command.arguments.is_empty() {
                    ListAllEffects::new(command.message)

                } else {
                    ListPatternEffects::new(command.message, command.arguments)
                }
            ]
        }
    }

}

