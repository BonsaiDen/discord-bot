// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server};
use super::Command;


// Command NotFound Message ---------------------------------------------------
pub struct NotFound {
    name: String
}


// Interface ------------------------------------------------------------------
impl NotFound {
    pub fn new(name: &str) -> NotFound {
        NotFound {
            name: name.to_string()
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for NotFound {

    fn execute(&self, _: &mut Handle, _: &mut Server) -> Option<Vec<String>> {
        info!("[Command] [NotFound] {}", self.name);
        Some(vec![format!(
            "The command `{}` does not exist, please type `!help` for a list of all available commands.",
            self.name
        )])
    }

    fn is_unique(&self) -> bool {
        false
    }

}

