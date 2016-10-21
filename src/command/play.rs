// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler {
    queued: bool
}

impl Handler {

    pub fn instant() -> Handler {
        Handler {
            queued: false
        }
    }

    pub fn queued() -> Handler {
        Handler {
            queued: true
        }
    }

}

impl CommandHandler for Handler {

    require_unique_server!();

    fn run(&self, command: Command) -> ActionGroup {
        if command.arguments.is_empty() {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!("Usage: `!{} <effect_name>`", if self.queued {
                    "q"

                } else {
                    "s"
                })
            ))

        } else {

            let effects = command.server.map_effects(
                &command.arguments[..],
                false,
                command.config
            );

            if effects.is_empty() {

                let similiar = command.server.map_similiar_effects(&command.arguments[..]);
                if similiar.is_empty() {
                    self.delete_and_send(command.message, MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.",
                            command.arguments.join("`, `"),
                            command.server.name
                        )
                    ))

                } else {
                    self.delete_and_send(command.message, MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.\n\n**Perhaps meant one of the following:**\n\n`{}`",
                            command.arguments.join("`, `"),
                            command.server.name,
                            similiar.join("`, `")
                        )
                    ))
                }

            } else if let Some(channel_id) = command.member.voice_channel_id {
                self.delete_and_send(command.message, EffectActions::Play::new(
                    command.message.server_id,
                    channel_id,
                    effects,
                    self.queued
                ))

            } else {
                self.delete_and_send(command.message, MessageActions::Send::private(
                    &command.message,
                    format!(
                        "You must be in a voice channel on {} in order to play sound effects.",
                        command.server.name
                    )
                ))
            }

        }

    }

}

