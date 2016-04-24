// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use rand::{thread_rng, Rng};


// Internal Dependencies ------------------------------------------------------
use super::super::super::util;
use super::Effect;


// Sound Effect Manager -------------------------------------------------------
pub struct EffectManager {

    // Effects
    effects: HashMap<String, Effect>,
    effects_directory: PathBuf,

    // Aliases
    aliases: HashMap<String, String>

}

impl EffectManager {

    pub fn new(effects_directory: PathBuf) -> EffectManager {
        EffectManager {

            // Effects
            effects: HashMap::new(),
            effects_directory: effects_directory,

            // Aliases
            aliases: HashMap::new()

        }
    }

    pub fn load_effects(&mut self) {

        // TODO pre-load all effects and calculate compression?
        self.effects.clear();
        self.aliases.clear();

        util::filter_dir(self.effects_directory.clone(), "flac", |name, path| {
            self.effects.insert(name.clone(), Effect::new(name, path));
        });

    }

    pub fn load_aliases(&mut self) {
        // TODO load for specific server
        self.aliases.clear();
    }

    pub fn map_from_patterns(&self, names: &[String]) -> Vec<Effect> {

        let effects: Vec<Effect> = names.iter()
             .map(|name| self.map_from_pattern(name))
             .filter(|e| e.is_some())
             .map(|e|e.unwrap())
             .collect();

        // TODO improve effects name listing
        info!(
            "[EffectManager] Mapped \"{}\" to \"{}\"",
            names.join("\", \""),
            effects.iter().map(|e| e.name.as_str() ).collect::<Vec<&str>>().join("\", \"")
        );
        effects

    }

    fn map_from_pattern(&self, pattern: &str) -> Option<Effect> {

        let matching: Vec<&str> = self.effects.keys().map(|n| {
            n.as_str()

        }).filter(|name| match_name_pattern(name, pattern) ).collect();

        if let Some(name) = thread_rng().choose(&matching[..]) {
            if let Some(effect) = self.effects.get(*name) {
                Some(effect.clone())

            } else {
                None
            }

        } else {
            None
        }

    }

}


// Helpers --------------------------------------------------------------------
fn match_name_pattern(name: &str, pattern: &str) -> bool {

    let len = pattern.len();

    // Random
    if pattern == "*" {
        true

    // Contains
    } else if len > 2 && pattern.starts_with('*') && pattern.ends_with('*') {
        name.contains(&pattern[1..len - 1])

    // Endswith
    } else if len > 1 && pattern.starts_with('*') {
        name.ends_with(&pattern[1..])

    // Startswith
    } else if len > 1 && pattern.ends_with('*') {
        name.starts_with(&pattern[0..len - 1])

    } else if len > 0 {

        // Exact
        if name == pattern {
            true

        // Prefix
        } else {
            // TODO optimize
            name.starts_with(&format!("{}_", pattern))
        }

    } else {
        false
    }

}

