// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl {
    queued: bool
}

impl CommandImpl {

    pub fn instant() -> CommandImpl {
        CommandImpl {
            queued: false
        }
    }

    pub fn queued() -> CommandImpl {
        CommandImpl {
            queued: true
        }
    }

}

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        server: &Server,
        member: &Member,
        config: &BotConfig

    ) -> ActionGroup {
        if !command.message.has_unique_server() {
            self.requires_unique_server(command)

        } else if command.arguments.is_empty() {
            self.delete_and_send(command.message, MessageActions::Send::private(
                &command.message,
                format!("Usage: `!{} <effect_name>`", if self.queued {
                    "q"

                } else {
                    "s"
                })
            ))

        } else {

            let effects = server.map_effects(
                &command.arguments[..],
                false,
                config
            );

            if effects.is_empty() {

                let similiar = server.map_similiar_effects(&command.arguments[..]);
                if similiar.is_empty() {
                    self.delete_and_send(command.message, MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.",
                            command.arguments.join("`, `"),
                            server.name
                        )
                    ))

                } else {
                    self.delete_and_send(command.message, MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.\n\n**Perhaps meant one of the following:**\n\n`{}`",
                            command.arguments.join("`, `"),
                            server.name,
                            similiar.join("`, `")
                        )
                    ))
                }

            } else if let Some(channel_id) = member.voice_channel_id {
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
                        server.name
                    )
                ))
            }

        }

    }

}

