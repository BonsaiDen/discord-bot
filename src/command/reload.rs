// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions, ServerActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        server: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        if !command.message.origin.is_unique() {
            self.requires_unique_server(command)

        } else {
            vec![
                ServerActions::Reload::new(command.message),
                MessageActions::Delete::new(command.message),
                MessageActions::Send::public(
                    &command.message,
                    format!(
                        "{} requested a configuration reload for {}.",
                        member.nickname,
                        server.name
                    )
                )
            ]
        }
    }

}

