// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


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
        if !command.message.origin.is_unique() {
            self.requires_unique_server(command)

        } else {
            vec![
                EffectActions::Silence::new(command.message),
                MessageActions::Delete::new(command.message),
                MessageActions::Send::public(
                    &command.message,
                    format!("{} has requested me to stay quiet.", member.nickname)
                )
            ]
        }
    }

}


