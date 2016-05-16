// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Server Silence Request -----------------------------------------------------
pub struct Silence;


// Command Implementation -----------------------------------------------------
impl Command for Silence {

    fn execute(&mut self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Silence] Silence requested.", server, user);
        server.request_silence();
        Some(vec![
            format!("{} has requested me to stay quiet.", user.nickname)
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

