// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server};
use super::Command;


// Command NotUnique Message ---------------------------------------------------
pub struct NotUnique {
    name: String
}


// Interface ------------------------------------------------------------------
impl NotUnique {
    pub fn new(name: &str) -> NotUnique {
        NotUnique {
            name: name.to_string()
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for NotUnique {

    fn execute(&self, _: &mut Handle, _: &mut Server) -> Option<Vec<String>> {
        info!("[Command] [NotUnique] {}", self.name);
        Some(vec![format!(
            "The command `{}` requires a unique server as its target, but you're a member of at least two different servers.
             Please re-issue the command from one of the public channels of the server you want to run it for.",
            self.name
        )])
    }

    fn is_unique(&self) -> bool {
        false
    }

}

