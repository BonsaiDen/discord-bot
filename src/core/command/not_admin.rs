// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Command NotAdmin Message ---------------------------------------------------
pub struct NotAdmin {
    name: String
}


// Interface ------------------------------------------------------------------
impl NotAdmin {
    pub fn new(name: &str) -> NotAdmin {
        NotAdmin {
            name: name.to_string()
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for NotAdmin {

    fn execute(&mut self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        info!("[{}] [{}] [Command] [NotAdmin] Admin is required for command \"{}\".", server, user, self.name);

        Some(vec![format!(
            "Sorry, but the command `{}` requires admin rights.",
            self.name
        )])

    }

    fn requires_unique_server(&self) -> bool {
        false
    }

    fn auto_remove_message(&self) -> bool {
        false
    }

    fn private_response(&self)-> bool {
        true
    }

}

