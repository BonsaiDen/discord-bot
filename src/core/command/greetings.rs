// Internal Dependencies ------------------------------------------------------
use super::super::super::util;
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Greetings Listing ----------------------------------------------------------
pub struct Greetings;


// Interface ------------------------------------------------------------------
impl Greetings {
    pub fn new() -> Greetings {
        Greetings
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Greetings {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [Greetings] User greeting listing requested.", server, user);
        let greetings = server.list_greetings();
        Some(util::list_lines("User Greetings", greetings, 100))
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

