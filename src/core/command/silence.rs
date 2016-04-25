// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Server Silence Request -----------------------------------------------------
pub struct Silence;


// Interface ------------------------------------------------------------------
impl Silence {
    pub fn new() -> Silence {
        Silence
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Silence {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Silence] Silence requested.", server, user);
        server.request_silence();
        None
    }

    fn is_server_unique(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        false
    }

}

