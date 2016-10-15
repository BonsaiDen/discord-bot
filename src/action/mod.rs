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
pub mod server;


// Re-Exports -----------------------------------------------------------------
pub use self::alias as AliasActions;
pub use self::ban as BanActions;
pub use self::effect as EffectActions;
pub use self::greeting as GreetingActions;
pub use self::message as MessageActions;
pub use self::server as ServerActions;


// General Action Abstraction -------------------------------------------------
pub type ActionGroup = Vec<Box<Action>>;

pub trait Action: fmt::Display {
    fn run(&self, &mut Bot, &BotConfig, &mut EventQueue) -> ActionGroup;
}

