// Internal Dependencies ------------------------------------------------------
use super::super::super::util;
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Aliases Listing ------------------------------------------------------------
pub struct Aliases;


// Command Implementation -----------------------------------------------------
impl Command for Aliases {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        info!("[{}] [{}] [Command] [Aliases] Effect alias listing requested.", server, user);

        let mut aliases = server.list_aliases();
        aliases.sort();

        if aliases.is_empty() {
            Some(vec![String::from("No effect aliases found for the current server.")])

        } else {
            Some(util::list_lines("Effect Aliases", aliases, 100))
        }

    }

    fn requires_unique_server(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        true
    }

}

