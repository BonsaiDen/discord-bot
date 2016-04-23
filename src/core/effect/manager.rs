// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;
use std::collections::HashMap;


// Internal Dependencies ------------------------------------------------------
use super::Effect;


// Sound Effect Manager -------------------------------------------------------
pub struct EffectManager {
    effects: HashMap<String, Effect>,
    aliases: HashMap<String, String>
}

impl EffectManager {

    pub fn new() -> EffectManager {
        EffectManager {
            effects: HashMap::new(),
            aliases: HashMap::new()
        }
    }

    pub fn load_from_directory(&mut self, _: PathBuf) {
        // TODO pre-load all effects and calculate compression?
        self.effects.clear();
        self.aliases.clear();
    }

    pub fn map_from_names(&mut self, _: &Vec<String>) -> Vec<Effect> {
        Vec::new()
    }

}

