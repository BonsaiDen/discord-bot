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
    effects: HashMap<String, Vec<Effect>>,
    effects_directory: PathBuf,

    // Aliases
    aliases: HashMap<String, Vec<Effect>>

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

    pub fn set_aliases(&mut self, aliases: HashMap<String, Vec<String>>) {

        self.aliases.clear();

        for (alias, names) in aliases {

            // TODO support * names in mappings
            let mapped: Vec<Effect> = names.iter().filter(|name| {
                self.effects.contains_key(*name)

            }).map(|name| {
                self.effects.get(name).unwrap().clone()

            }).flat_map(|s| s).collect();

            if !mapped.is_empty() {
                self.aliases.insert(alias, mapped);
            }

        }

    }

    pub fn get_aliases(&self) -> &HashMap<String, Vec<Effect>> {
        &self.aliases
    }

    pub fn load_effects(&mut self) {

        self.effects.clear();

        util::filter_dir(self.effects_directory.clone(), "flac", |name, path| {
            self.effects.insert(name.clone(), vec![Effect::new(name, path)]);
        });

    }

    pub fn list_effects(&self) -> Vec<&str> {
        self.effects.keys().map(|effect| {
            effect.as_str()

        }).collect()
    }

    pub fn map_from_patterns(&self, names: &[String]) -> Vec<Effect> {

        let effects: Vec<Effect> = names.iter()
             .map(|name| self.map_from_pattern(name))
             .filter(|e| e.is_some())
             .map(|e|e.unwrap())
             .flat_map(|s| s).collect();

        // TODO improve effects name listing
        info!(
            "[EffectManager] Mapped \"{}\" to \"{}\"",
            names.join("\", \""),
            effects.iter().map(|e| {
                e.name.as_str()

            }).collect::<Vec<&str>>().join("\", \"")
        );

        effects

    }

    fn map_from_pattern(&self, pattern: &str) -> Option<Vec<Effect>> {

        let mut matching: Vec<&str> = self.effects.keys().map(|n| {
            n.as_str()

        }).filter(|name| match_name_pattern(name, pattern) ).collect();

        let matching_aliases: Vec<&str> = self.aliases.keys().map(|n| {
            n.as_str()

        }).filter(|name| match_name_pattern(name, pattern) ).collect();

        matching.extend(matching_aliases);

        if let Some(name) = thread_rng().choose(&matching[..]) {
            if let Some(effect) = self.effects.get(*name) {
                Some(effect.clone())

            } else if let Some(effect) = self.aliases.get(*name) {
                Some(effect.clone())

            } else {
                None
            }

        } else {
            None
        }

    }

    pub fn download_effect(&mut self, effect: &str, url: &str) -> Result<(), String> {
        util::download_file(
            self.effects_directory.clone(),
            effect, "flac", url

        ).map_err(|err| err.to_string()).and_then(|_| Ok(self.load_effects()))
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

