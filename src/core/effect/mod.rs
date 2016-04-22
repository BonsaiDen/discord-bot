// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;


// Modules --------------------------------------------------------------------
mod manager;


// Re-Exports -----------------------------------------------------------------
pub use self::manager::EffectManager;


// Effect Abstraction ---------------------------------------------------------
pub struct Effect {
    name: String,
    path: PathBuf
}

