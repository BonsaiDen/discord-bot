// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::command::{Command, CommandHandler};
use ::core::{Member, MessageOrigin, Server};
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
        if command.message.origin == MessageOrigin::DirectMessage {
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
                // TODO provide a listing of similiar sounds
                self.delete_and_send(command.message, MessageActions::Send::private(
                    &command.message,
                    format!(
                        "No effect(s) matching `{}` found on {}.",
                        command.arguments.join("`, `"),
                        server.name
                    )
                ))

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

