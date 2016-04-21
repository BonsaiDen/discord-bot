// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


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

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        info!("[{}] [{}] [Command] [NotUnique] Not unique target for command \"{}\".", server, user, self.name);

        Some(vec![format!(
            "The command `{}` requires a unique server as its target, but you're a member of at least two different servers.
             Please re-issue the command from one of the public channels of the server you want to run it for.",
            self.name
        )])

    }

    fn is_unique(&self) -> bool {
        false
    }

    fn auto_remove(&self) -> bool {
        false
    }

}

