// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::Command;


// Server Configuration Reload ------------------------------------------------
pub struct Reload {
}


// Interface ------------------------------------------------------------------
impl Reload {
    pub fn new(_: Vec<&str>) -> Reload {
        Reload {
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Reload {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> Option<Vec<String>> {
        info!("[Command] [{}] [Reload] Server configuration reloaded.", server);
        Some(vec!["Server configuration reloaded.".to_string()])
    }

    fn is_unique(&self) -> bool {
        true
    }

}

