// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server};
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

    fn execute(&self, _: &mut Handle, server: &mut Server) -> Option<Vec<String>> {
        info!("[Command] [Reload] Configuration updated for {}", server);
        Some(vec!["Configuration reloaded.".to_string()])
    }

    fn is_unique(&self) -> bool {
        true
    }

}

