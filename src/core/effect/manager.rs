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
    aliases: HashMap<String, Vec<String>>

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

}


// Aliases --------------------------------------------------------------------
impl EffectManager {

    pub fn set_aliases(&mut self, aliases: HashMap<String, Vec<String>>) {
        self.aliases = aliases;
    }

    pub fn get_aliases(&self) -> &HashMap<String, Vec<String>> {
        &self.aliases
    }

    pub fn add_alias(&mut self, name: &str, effects: Vec<String>) {
        self.aliases.insert(name.to_string(), effects);
    }

    pub fn remove_alias(&mut self, name: &str) -> Option<Vec<String>> {
        self.aliases.remove(name)
    }

}


// Effects --------------------------------------------------------------------
impl EffectManager {

    pub fn load_effects(&mut self) {

        self.effects.clear();

        util::filter_dir(self.effects_directory.clone(), "flac", |name, path| {
            let mut split = name.split('.');
            let name = split.next().unwrap().to_string();
            self.effects.insert(name.clone(), vec![
                Effect::new(name, path)
            ]);
        });

    }

    pub fn list_effects(&self) -> Vec<&str> {
        self.effects.keys().map(|effect| effect.as_str()).collect()
    }

    pub fn download_effect(&mut self, effect: &str, user_ident: &str, url: &str) -> Result<(), String> {
        util::download_file(
            self.effects_directory.clone(),
            effect, user_ident, "flac", url

        ).map_err(|err| err.to_string()).and_then(|_| Ok(self.load_effects()))
    }

    pub fn delete_effect(&mut self, effect: &str) -> Result<(), String> {

        let mut filepath = None;
        util::filter_dir(self.effects_directory.clone(), "flac", |name, path| {

            let mut split = name.split('.');
            let name = split.next().unwrap().to_string();
            if name == effect {
                filepath = Some(path);
            }

        });

        if let Some(path) = filepath {
            util::delete_file(path).and_then(|_| {
                self.effects.remove(&effect.to_string());
                Ok(())
            })

        } else {
            Err("No matching effect file found.".to_string())
        }

    }

    pub fn rename_effect(&mut self, effect: &str, to: &str) -> Result<(), String> {

        let mut filepath = None;
        let mut new_filepath = PathBuf::new();
        util::filter_dir(self.effects_directory.clone(), "flac", |name, path| {

            let mut split = name.split('.');
            let name = split.next().unwrap().to_string();
            if name == effect {
                filepath = Some(path.clone());
                new_filepath = path;
                new_filepath.set_file_name(format!(
                    "{}.{}",
                    to,
                    split.collect::<Vec<&str>>().join(".")
                ));
                new_filepath.set_extension("flac");
            }

        });

        if let Some(path) = filepath {
            util::rename_file(path, new_filepath).and_then(|_| {
                self.load_effects();
                Ok(())
            })

        } else {
            Err("No matching effect file found.".to_string())
        }

    }


}


// Mapping --------------------------------------------------------------------
impl EffectManager {

    pub fn map_from_patterns(&self, patterns: &[String], all: bool) -> Vec<Effect> {

        let effects: Vec<Effect> = patterns.iter()
             .map(|name| self.map_from_pattern(name,  all))
             .filter(|e| e.is_some())
             .map(|e|e.unwrap())
             .flat_map(|s| s).collect();

        info!(
            "[EffectManager] Mapped \"{}\" to \"{}\"",
            patterns.join("\", \""),
            effects.iter().map(|e| {
                e.name.as_str()

            }).collect::<Vec<&str>>().join("\", \"")
        );

        effects

    }

    fn map_from_pattern(&self, pattern: &str, all: bool) -> Option<Vec<Effect>> {

        let mut matching: Vec<&str> = self.effects.keys().map(|n| {
            n.as_str()

        }).filter(|name| match_name_pattern(name, pattern) ).collect();

        let matching_aliases: Vec<&str> = self.aliases.keys().map(|n| {
            n.as_str()

        }).filter(|name| match_name_pattern(name, pattern) ).collect();

        matching.extend(matching_aliases);

        if all {
            let mut effects = Vec::new();
            for m in matching {
                if let Some(e) = self.effects.get(m) {
                    effects.append(&mut e.clone())
                }
            }
            Some(effects)

        } else if let Some(name) = thread_rng().choose(&matching[..]) {
            if let Some(effect) = self.effects.get(*name) {
                Some(effect.clone())

            } else if let Some(aliases) = self.aliases.get(*name) {
                Some(self.map_from_patterns(aliases, false))

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

