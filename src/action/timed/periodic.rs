// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup, TimedActions};


// Periodic Action Implementation ---------------------------------------------
pub struct Action {
    delay_millis: u64,
    action: Option<Box<ActionHandler>>
}


impl Action {

    pub fn new(delay_millis: u64, action: Box<ActionHandler>) -> Box<Action> {
        Box::new(Action {
            delay_millis: delay_millis,
            action: Some(action)
        })
    }

}

impl ActionHandler for Action {

    fn run(&mut self, bot: &mut Bot, bot_config: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        if let Some(mut action) = self.action.take() {
            let mut actions = action.run(bot, bot_config, queue);
            actions.push(TimedActions::Delayed::new(
                self.delay_millis,
                Action::new(self.delay_millis, action))
            );
            actions

        } else {
            vec![]
        }
    }

}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[PeriodicAction] {}", self.action.as_ref().unwrap())
    }
}

