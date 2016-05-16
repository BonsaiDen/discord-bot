// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Playback -------------------------------------------------------------
pub struct Sound {
    effect_names: Vec<String>,
    immediate: bool
}


// Interface ------------------------------------------------------------------
impl Sound {
    pub fn new(effect_names: Vec<&str>, immediate: bool) -> Sound {
        Sound {
            effect_names: effect_names.iter().map(|s| s.to_string()).collect(),
            immediate: immediate
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Sound {

    fn execute(&mut self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        info!(
            "[{}] [{}] [Command] [Sound] Playback requested for {} sounds(s) (immediate={}).",
            server, user, self.effect_names.len(), self.immediate
        );

        if self.effect_names.is_empty() {
            Some(vec![
                "You must specify at least one sound effect.".to_string()
            ])

        } else if let Some(channel_id) = handle.find_voice_channel_id_for_user(&user.id) {

            let effects = server.map_effects(&self.effect_names);
            if effects.is_empty() {

                let suggestions = server.get_effect_suggestions(
                    self.effect_names.get(0).unwrap(),
                    6, 5
                );

                if suggestions.is_empty() {
                    Some(vec![
                        format!(
                            "None of the requested sound effect(s) `{}` exist. \
                            See `!sounds` for a list of all available effects.",
                            self.effect_names.join("`, `")
                        )
                    ])

                } else {
                    Some(vec![
                        format!(
                            "None of the requested sound effect(s) `{}` exist. \
                            Did you mean one of `{}`?  \
                            If not, see `!sounds` for a list of all available effects.",
                            self.effect_names.join("`, `"),
                            suggestions.join("`, `")
                        )
                    ])
                }

            } else {
                server.play_effects(handle, channel_id, effects, self.immediate, 0);
                None
            }

        } else {
            Some(vec![
                 "You must be in a voice channel in order to play sound effects.".to_string()
            ])
        }

    }

    fn requires_unique_server(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        true
    }

}

