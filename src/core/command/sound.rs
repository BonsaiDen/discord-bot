// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Playback -------------------------------------------------------------
pub struct Sound {
    effects: Vec<String>,
    immediate: bool
}


// Interface ------------------------------------------------------------------
impl Sound {
    pub fn new(effects: Vec<&str>, immediate: bool) -> Sound {
        Sound {
            effects: effects.iter().map(|s| s.to_string()).collect(),
            immediate: immediate
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Sound {

    fn execute(&self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        info!(
            "[{}] [{}] [Command] [Sound] Sound playback requested (immediate={}): {}.",
            server, user, self.immediate, self.effects.join(", ")
        );

        if let Some(channel_id) = handle.find_voice_channel_id_for_user(&user.id) {
            server.join_voice_channel(handle, Some(channel_id));
            None

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

