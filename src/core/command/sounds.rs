// Internal Dependencies ------------------------------------------------------
use super::super::super::util;
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Listing --------------------------------------------------------------
pub struct Sounds;


// Interface ------------------------------------------------------------------
impl Sounds {
    pub fn new() -> Sounds {
        Sounds
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Sounds {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Sounds] Sound listing requested.", server, user);
        let mut effects = server.list_effects();
        effects.sort();
        Some(util::list_words("Sound Effects", effects, 100, 4))
    }

    fn is_unique(&self) -> bool {
        true
    }

    fn auto_remove(&self) -> bool {
        true
    }

}

