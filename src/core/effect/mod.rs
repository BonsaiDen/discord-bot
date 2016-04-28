// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::path::PathBuf;


// Modules --------------------------------------------------------------------
mod manager;


// Re-Exports -----------------------------------------------------------------
pub use self::manager::EffectManager;


// Effect Abstraction ---------------------------------------------------------
#[derive(Clone)]
pub struct Effect {
    name: String,
    path: PathBuf
}

impl Effect {

    pub fn new(name: String, path: PathBuf) -> Effect {
        Effect {
            name: name,
            path: path
        }
    }

    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Effect {} @ {}]", self.name, self.path.to_str().unwrap_or(""))
    }
}


