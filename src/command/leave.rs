// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, SendPublicMessage, LeaveServerVoice};


// Command Implementation -----------------------------------------------------
pub struct LeaveCommand;

impl CommandImplementation for LeaveCommand {

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
                LeaveServerVoice::new(command.message),
                DeleteMessage::new(command.message),
                SendPublicMessage::new(
                    &command.message,
                    format!("{} has requested me to leave", member.nickname)
                )
            ]
        }
    }

}

