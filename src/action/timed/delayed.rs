// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Delayed Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    delay_until: u64,
    action: Option<Box<ActionHandler>>
}


impl ActionImpl {

    fn new(delay_millis: u64, action: Box<ActionHandler>) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            delay_until: clock_ticks::precise_time_ms() + delay_millis,
            action: Some(action)
        })
    }

}

impl ActionHandler for ActionImpl {

    fn ready(&self) -> bool {
        clock_ticks::precise_time_ms() > self.delay_until
    }

    fn run(&mut self, _: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {
        if let Some(action) = self.action.take() {
            vec![action]

        } else {
            vec![]
        }
    }

}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[DelayedAction] {}", self.action.as_ref().unwrap())
    }
}

