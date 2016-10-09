// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::MessageOrigin;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, PlayEffects, SendPublicMessage};


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
            vec![
                DeleteMessage::new(command.message),
                SendPublicMessage::new(
                    &command.message,
                    format!("Usage: `!{} <effect_name>`", if self.queued {
                        "q"

                    } else {
                        "s"
                    })
                )
            ]

        } else {

            let effects = server.map_effects(
                &command.arguments[..],
                false,
                config
            );

            if effects.is_empty() {
                vec![
                    DeleteMessage::new(command.message),
                    SendPublicMessage::new(
                        &command.message,
                        "No matching effects found".to_string()
                    )
                ]

            } else if let Some(channel_id) = member.voice_channel_id {
                vec![
                    DeleteMessage::new(command.message),
                    PlayEffects::new(
                        command.message.server_id,
                        channel_id,
                        effects,
                        self.queued
                    )
                ]

            } else {
                vec![
                    DeleteMessage::new(command.message),
                    SendPublicMessage::new(
                        &command.message,
                        "You must be in a voice channel in order to play sound effects.".to_string()
                    )
                ]
            }

        }

    }

}

