// STD Dependencies -----------------------------------------------------------
use std::fmt;
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

impl Effect {
    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }
}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Effect {} @ {}]", self.name, self.path.to_str().unwrap_or(""))
    }
}


