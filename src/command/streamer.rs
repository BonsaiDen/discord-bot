// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, TwitchActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_min_arguments!(2);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        match command.arguments[0].as_str() {
            "add" => if command.arguments.len() < 3 {
                self.usage(command)

            } else {
                self.add(
                    &command,
                    &command.arguments[1],
                    &command.arguments[2]
                )
            },
            "remove" => self.remove(
                &command,
                &command.arguments[1],
            ),
            _ => self.usage(command)
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!streamer add <twitch_nick> <channel_name>` or `!streamer remove <twitch_nick>`".to_string()
        )
    }

}

impl Handler {

    fn add(
        &self,
        command: &Command,
        twitch_nick: &str,
        channel_name: &str

    ) -> ActionGroup {
        if command.server.has_streamer(twitch_nick) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "A twitch streamer named `{}` is already being watched on {}.",
                    twitch_nick, command.server.name
                )
            )

        } else {
            vec![TwitchActions::AddStreamer::new(
                command.message,
                twitch_nick.to_string(),
                channel_name.to_string()
            )]
        }
    }

    fn remove(&self, command: &Command, twitch_nick: &str) -> ActionGroup {
        if !command.server.has_streamer(twitch_nick) {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "A twitch streamer named `{}` is not being watched on {}.",
                    twitch_nick, command.server.name
                )
            )

        } else {
            vec![TwitchActions::RemoveStreamer::new(command.message, twitch_nick.to_string())]
        }
    }

}

