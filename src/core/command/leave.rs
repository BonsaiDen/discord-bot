// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Server Leave Voice Request -------------------------------------------------
pub struct Leave;


// Command Implementation -----------------------------------------------------
impl Command for Leave {

    fn execute(&self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Leave] Voice leave requested.", server, user);
        server.leave_voice_channel(handle);
        Some(vec![
            format!("{} has requested me to leave.", user.nickname)
        ])
    }

    fn requires_unique_server(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        false
    }

}


