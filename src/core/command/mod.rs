// Internal Dependencies ------------------------------------------------------
use super::{Handle, Server, User};


// Commands -------------------------------------------------------------------
mod reload;
mod not_found;
mod not_unique;


// Type Aliases ---------------------------------------------------------------
pub type CommandResult = Option<Vec<String>>;


// Module Interface -----------------------------------------------------------
pub fn from_args(
    name: &str,
    arguments: Vec<&str>,
    unique_server: bool

) -> Box<Command> {

    let command = match_from_args(name, arguments);
    if !unique_server && command.is_unique() {
        Box::new(not_unique::NotUnique::new(name))

    } else {
        command
    }

}

fn match_from_args(name: &str, arguments: Vec<&str>) -> Box<Command> {

    info!("[Command] Matching \"{}\" with arguments \"{}\"...", name, arguments.join("\", \""));

    // Match command and arguments
    match name {
        "reload" => Box::new(reload::Reload::new(arguments)),
        _ => Box::new(not_found::NotFound::new(name))
    }

}


// Traits ---------------------------------------------------------------------
pub trait Command {
    fn execute(&self, &mut Handle, &mut Server, &User) -> CommandResult;
    fn is_unique(&self) -> bool;
    fn auto_remove(&self) -> bool;
}

