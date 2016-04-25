// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Server Configuration Reload ------------------------------------------------
pub struct Reload;


// Interface ------------------------------------------------------------------
impl Reload {
    pub fn new() -> Reload {
        Reload
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Reload {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Reload] Server configuration reloaded.", server, user);
        server.reload_configuration();
        Some(vec![
            format!("{} requested a configuration reload for this server.", user.nickname)
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

