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

    fn execute(&self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

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
            if !effects.is_empty() {
                server.play_effects(handle, channel_id, effects, self.immediate, 0);
                None

            } else {
                server.play_effects(handle, channel_id, effects, self.immediate, 0);
                None
                //Some(vec![
                //    format!(
                //        "None of the requested sound effect(s) `{}` exist. \
                //        Please see `!sounds` for a list of all available effects.",
                //        self.effect_names.join("`, `")
                //    )
                //])
            }

        } else {
            Some(vec![
                 "You must be in a voice channel in order to play sound effects.".to_string()
            ])
        }

    }

    fn is_unique(&self) -> bool {
        true
    }

    fn auto_remove(&self) -> bool {
        true
    }

}

