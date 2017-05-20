// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Statics --------------------------------------------------------------------
static USAGE_TEXT_S: &str = "Usage: `!s <effect_name>, ...`

Instantly starts the playback of one or more requested sound effects.

Each **`effect_name`** can be one of the following patterns:

- `full_sound_name` - Only the exactly matching effect.
- `prefix` - A random effect which name starts with the specified prefix, followed by an underscore.
- `*wildcard` - A random effect which *ends* with the specified wildcard.
- `wildcard*` - A random effect which *starts* with the specified wildcard.
- `*wildcard*` - A random effect which *contains* the specified wildcard.

If more than one effect is requested, a playback queue will be created and the effects will be played back one after another.

If another queue is created via the `!s` command while the previous one is still active, at most two effects will be played simultaneously.";

static USAGE_TEXT_Q: &str = "Usage: `!q <effect_name>, ...`

Queues starts the playback of one or more requested sound effects.

*Playback will only start once all other currently playing / requested sound effects have finished.*

Each **`effect_name`** can be one of the following patterns:

- `full_sound_name` - Only the exactly matching effect.
- `prefix` - A random effect which name starts with the specified prefix, followed by an underscore.
- `*wildcard` - A random effect which *ends* with the specified wildcard.
- `wildcard*` - A random effect which *starts* with the specified wildcard.
- `*wildcard*` - A random effect which *contains* the specified wildcard.

If more than one effect is requested, a playback queue will be created and the effects will be played back one after another.

If another queue is created via the `!q` command while the previous one is still active, at most two effects will be played simultaneously.";


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
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        if command.arguments.is_empty() {
            MessageActions::Send::private(
                &command.message,
                format!("Usage: `!{} <effect_name>`", if self.queued {
                    "q"

                } else {
                    "s"
                })
            )

        } else {

            let effects = command.server.map_effects(
                &command.arguments[..],
                false,
                command.config
            );

            if effects.is_empty() {

                let similiar = command.server.map_similiar_effects(&command.arguments[..]);
                if similiar.is_empty() {
                    MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.",
                            command.arguments.join("`, `"),
                            command.server.name
                        )
                    )

                } else {
                    MessageActions::Send::private(
                        &command.message,
                        format!(
                            "No effect(s) matching `{}` were found on {}.\n\n**Perhaps you meant one of the following:**\n\n`{}`",
                            command.arguments.join("`, `"),
                            command.server.name,
                            similiar.join("`, `")
                        )
                    )
                }

            } else if let Some(channel_id) = command.member.voice_channel_id {
                vec![EffectActions::Play::new(
                    command.message.server_id,
                    channel_id,
                    effects,
                    self.queued,
                    None
                )]

            } else {
                MessageActions::Send::private(
                    &command.message,
                    format!(
                        "You must be in a voice channel on {} in order to play sound effects.",
                        command.server.name
                    )
                )
            }

        }

    }

    fn help(&self) -> &str {
        if self.queued {
            "Queue a sound effect to be played in current voice channel."

        } else {
            "Instantly play a sound effect in your current voice channel."
        }
    }

    fn usage(&self, command: Command) -> ActionGroup {
        if self.queued {
            MessageActions::Send::private(&command.message, USAGE_TEXT_Q.to_string())

        } else {
            MessageActions::Send::private(&command.message, USAGE_TEXT_S.to_string())
        }
    }

}

