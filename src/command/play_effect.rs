// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, PlayEffects, SendPublicMessage};


// Command Implementation -----------------------------------------------------
pub struct PlayEffectCommand {
    queued: bool
}

impl PlayEffectCommand {

    pub fn instant() -> PlayEffectCommand {
        PlayEffectCommand {
            queued: false
        }
    }

    pub fn queued() -> PlayEffectCommand {
        PlayEffectCommand {
            queued: true
        }
    }

}

impl CommandImplementation for PlayEffectCommand {

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
            self.delete_and_send(command.message, SendPublicMessage::new(
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
                self.delete_and_send(command.message, SendPublicMessage::new(
                    &command.message,
                    format!("No matching effect(s) found on {}.", server.name)
                ))

            } else if let Some(channel_id) = member.voice_channel_id {
                self.delete_and_send(command.message, PlayEffects::new(
                    command.message.server_id,
                    channel_id,
                    effects,
                    self.queued
                ))

            } else {
                self.delete_and_send(command.message, SendPublicMessage::new(
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

