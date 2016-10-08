// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{
    ActionGroup, DeleteMessage, SendPublicMessage, ReloadServerConfiguration
};


// Command Implementation -----------------------------------------------------
pub struct ReloadCommand;

impl CommandImplementation for ReloadCommand {

    fn run(
        &self,
        command: Command,
        _: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if command.message.origin == MessageOrigin::DirectMessage {
            self.requires_unique_server(command)

        } else {
            vec![
                ReloadServerConfiguration::new(command.message),
                DeleteMessage::new(command.message),
                SendPublicMessage::new(
                    &command.message,
                    format!(
                        "{} requested a configuration reload for this server.",
                        member.nickname
                    )
                )
            ]
        }
    }

}

