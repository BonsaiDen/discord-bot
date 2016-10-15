// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions};
use ::core::{Member, MessageOrigin, Server};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

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
                EffectActions::List::all(command.message)

            } else {
                EffectActions::List::matching(command.message, command.arguments)
            })
        }
    }

}

