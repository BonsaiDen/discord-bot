// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, ListBans};


// Command Implementation -----------------------------------------------------
pub struct BansCommand;

impl CommandImplementation for BansCommand {

    fn run(
        &self,
        command: Command,
        _: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if command.message.origin == MessageOrigin::DirectMessage {
            self.requires_unique_server(command)

        } else if !member.is_admin {
            self.requires_admin(command)

        } else {
            vec![
                DeleteMessage::new(command.message),
                ListBans::new(command.message)
            ]
        }
    }

}

