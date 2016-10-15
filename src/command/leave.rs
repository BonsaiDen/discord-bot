// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::command::{Command, CommandHandler};
use ::core::{Member, MessageOrigin, Server};
use ::action::{ActionGroup, MessageActions, ServerActions};


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
        if command.message.origin == MessageOrigin::DirectMessage {
            self.requires_unique_server(command)

        } else {
            vec![
                ServerActions::LeaveVoice::new(command.message),
                MessageActions::Delete::new(command.message),
                MessageActions::Send::public(
                    &command.message,
                    format!(
                        "{} has requested me to leave the voice channel.",
                        member.nickname
                    )
                )
            ]
        }
    }

}

