// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::core::EventQueue;
use ::bot::{Bot, BotConfig};


// Modules --------------------------------------------------------------------
pub mod alias;
pub mod ban;
pub mod effect;
pub mod greeting;
pub mod message;
pub mod recording;
pub mod server;
pub mod timed;


// Re-Exports -----------------------------------------------------------------
pub use self::alias as AliasActions;
pub use self::ban as BanActions;
pub use self::effect as EffectActions;
pub use self::greeting as GreetingActions;
pub use self::message as MessageActions;
pub use self::recording as RecordingActions;
pub use self::server as ServerActions;
pub use self::timed::Delayed as DelayedAction;


// General Action Abstraction -------------------------------------------------
pub type ActionGroup = Vec<Box<ActionHandler>>;

pub trait ActionHandler: fmt::Display {

    fn ready(&self) -> bool {
        true
    }

    fn run(&mut self, &mut Bot, &BotConfig, &mut EventQueue) -> ActionGroup;

}

