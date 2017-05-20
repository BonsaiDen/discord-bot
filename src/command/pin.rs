// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions, ServerActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {

        if !command.server.is_in_voice() {
            MessageActions::Send::private(
                &command.message,
                "Can only pin when actually in a voice channel.".to_string()
            )

        } else {
            vec![
                ServerActions::PinVoice::new(command.message),
                MessageActions::Send::single_public(
                    &command.message,
                    format!(
                        "{} has pinned me to my current voice channel. Use `!leave` to unpin me.",
                        command.member.nickname
                    )
                )
            ]
        }

    }

    fn help(&self) -> &str {
        "Pin the bot onto the current channel."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!pin`".to_string()
        )
    }

}

