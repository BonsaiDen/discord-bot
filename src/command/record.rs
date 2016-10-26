// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions, RecordingActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    require_server_admin!();
    require_exact_arguments!(1);
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        match command.arguments[0].as_str() {
            "start" => self.start(&command),
            "stop" => self.stop(&command),
            _ => self.usage(command)
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::public(
            &command.message,
            "Usage: `!record [start|stop]`".to_string()
        )
    }

}

impl Handler {

    fn start(&self, command: &Command) -> ActionGroup {
        if command.server.is_recording_voice() {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "Audio is already being recorded on {}",
                    command.server.name
                )
            )

        } else if let Some(channel_id) = command.member.voice_channel_id {
            vec![RecordingActions::Start::new(
                command.message.server_id,
                channel_id
            )]

        } else {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "You must be in a voice channel on {} in order start audio recording.",
                    command.server.name
                )
            )
        }
    }

    fn stop(&self, command: &Command) -> ActionGroup {
        if !command.server.is_recording_voice() {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "Audio is currently not being recorded on {}.",
                    command.server.name
                )
            )

        } else {
            vec![RecordingActions::Stop::new(command.message.server_id)]
        }
    }

}

