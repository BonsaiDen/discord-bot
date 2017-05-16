// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// Log Action Implementation -------------------------------------------------
pub struct Action {
    message: String
}


impl Action {

    pub fn new(message: String) -> Box<Action> {
        Box::new(Action {
            message: message
        })
    }

}

impl ActionHandler for Action {

    fn run(&mut self, _: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        info!("{}", self.message);
        vec![]
    }

}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[LogAction] {}", self.message)
    }
}


