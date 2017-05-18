// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use clock_ticks;
use rand::{thread_rng, Rng};
use edit_distance::edit_distance;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::effect::Effect;
use ::server::ServerConfig;


// Modules --------------------------------------------------------------------
mod manage;


// Effects Registration -------------------------------------------------------
#[derive(Debug)]
pub struct EffectRegistry {
    effects: HashMap<String, Effect>,
    last_played: HashMap<String, u64>
}


// Public Interface -----------------------------------------------------------
impl EffectRegistry {

    pub fn new() -> EffectRegistry {
        EffectRegistry {
            effects: HashMap::new(),
            last_played: HashMap::new()
        }
    }

    pub fn reload(&mut self, config: &ServerConfig) {
        self.reload_effects(config);
    }

    pub fn has_effect(&self, name: &str) -> bool {
        self.effects.contains_key(name)
    }

    pub fn get_effect(&self, name: &str) -> Option<&Effect> {
        self.effects.get(name)
    }

    pub fn played_effect(&mut self, name: &str) {
        self.last_played.insert(
            name.to_string(),
            clock_ticks::precise_time_ms()
        );
    }

    pub fn map_patterns(
        &self,
        patterns: &[String],
        aliases: Option<&HashMap<String, Vec<String>>>,
        match_all: bool,
        config: &BotConfig

    ) -> Vec<&Effect> {

        let effects: Vec<&Effect> = patterns.iter()
             .map(|name| self.map_from_pattern(name, aliases, match_all, config))
             .filter_map(|e| e)
             .flat_map(|s| s).collect();

        info!(
            "{} Mapped \"{}\" to \"{}\"",
            self,
            patterns.join("\", \""),
            effects.iter().map(|e| {
                e.name.as_str()

            }).collect::<Vec<&str>>().join("\", \"")
        );

        effects

    }

    pub fn map_similiar(
        &self,
        patterns: &[String]

    ) -> Vec<&str> {
        self.effects.keys().map(|name| name.as_str()).filter(|name| {
            patterns.iter().any(|p| {

                let len = p.len();
                let p = if len > 2 && p.starts_with('*') && p.ends_with('*') {
                    &p[1..len - 1]

                } else if len > 1 && p.starts_with('*') {
                    &p[1..]

                } else if len > 1 && p.ends_with('*') {
                    &p[0..len - 1]

                } else {
                    p
                };

                name.ends_with(p) || name.starts_with(p)
                    || name.contains(p) || edit_distance(name, p) < 3

            })

        }).take(10).collect()
    }

}


// Internal Interface ---------------------------------------------------------
impl EffectRegistry {

    fn map_from_pattern(
        &self,
        pattern: &str,
        aliases: Option<&HashMap<String, Vec<String>>>,
        match_all: bool,
        config: &BotConfig

    ) -> Option<Vec<&Effect>> {

        // Find matching effect names
        let mut matching_effects: Vec<&str> = self.effects.values().filter(|effect| {
            match_effect_pattern(
                effect,
                *self.last_played.get(&effect.name).unwrap_or(&0),
                pattern,
                match_all,
                config.effect_playback_separation_ms
            )

        }).map(|effect| {
            effect.name.as_str()

        }).collect();

        // Find matching alias names, if provided
        if let Some(aliases) = aliases {

            let matching_aliases: Vec<&str> = aliases.keys().map(|n| {
                n.as_str()

            }).filter(|name| {
                match_alias_pattern(name, pattern)

            }).collect();

            matching_effects.extend(matching_aliases);

        }

        if match_all {
            let mut effects = Vec::new();
            for m in matching_effects {
                if let Some(e) = self.effects.get(m) {
                    effects.push(e)
                }
            }
            Some(effects)

        } else {
            self.map_random_effect(matching_effects, aliases, config)
        }

    }

    fn map_random_effect(
        &self,
        effects: Vec<&str>,
        aliases: Option<&HashMap<String, Vec<String>>>,
        bot_config: &BotConfig

    ) -> Option<Vec<&Effect>> {

        // Select one random effect...
        if let Some(name) = thread_rng().choose(&effects[..]) {

            // ...selected effect is already an actual effect
            if let Some(effect) = self.effects.get(*name) {
                Some(vec![effect])

            // ...selected effect is an alias, so we need to resolve its mapped effect
            } else if let Some(aliases) = aliases {
                if let Some(effects) = aliases.get(*name) {
                    Some(self.map_patterns(effects, None, false, bot_config))

                } else {
                    None
                }

            } else {
                None
            }

        } else {
            None
        }

    }


}


// Traits ---------------------------------------------------------------------
impl fmt::Display for EffectRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[EffectRegistry with {} effects(s)]",
            self.effects.len()
        )
    }
}


// Helpers --------------------------------------------------------------------
fn match_effect_pattern(
    effect: &Effect,
    last_played: u64,
    pattern: &str,
    ignore_recent: bool,
    recent_threshold: u64

) -> bool {

    let len = pattern.len();

    // Name: Random
    if pattern == "*" {
        ignore_recent || !was_recently_played(last_played, recent_threshold)

    // Name: Contains
    } else if len > 2 && pattern.starts_with('*') && pattern.ends_with('*') {
        effect.name.contains(&pattern[1..len - 1])

    // Transcript: Contains
    } else if len > 2 && pattern.starts_with('"') && pattern.ends_with('"') {
        effect.transcript().contains(&pattern[1..len - 1].to_string())

    // Name: Endswith
    } else if len > 1 && pattern.starts_with('*') {
        effect.name.ends_with(&pattern[1..])

    // Name: Startswith
    } else if len > 1 && pattern.ends_with('*') {
        effect.name.starts_with(&pattern[0..len - 1])

    } else if len > 0 {

        // Name: Exact
        if effect.name == pattern {
            true

        // Name: Prefix
        } else if effect.name.starts_with(&format!("{}_", pattern)) {
            ignore_recent || !was_recently_played(last_played, recent_threshold)

        } else {
            false
        }

    } else {
        false
    }

}

fn was_recently_played(last_played: u64, threshold: u64) -> bool {
    last_played + threshold > clock_ticks::precise_time_ms()
}

fn match_alias_pattern(alias: &str, pattern: &str) -> bool {

    let len = pattern.len();

    // Random
    if pattern == "*" {
        true

    // Name: Contains
    } else if len > 2 && pattern.starts_with('*') && pattern.ends_with('*') {
        alias.contains(&pattern[1..len - 1])

    // Name: Endswith
    } else if len > 1 && pattern.starts_with('*') {
        alias.ends_with(&pattern[1..])

    // Name: Startswith
    } else if len > 1 && pattern.ends_with('*') {
        alias.starts_with(&pattern[0..len - 1])

    // Name: Exact or Prefix
    } else if len > 0 {
        alias == pattern || alias.starts_with(&format!("{}_", pattern))

    } else {
        false
    }

}

/*
fn load_transcript(mut flac_path: PathBuf) -> Option<String> {

    flac_path.set_extension("txt");

    if let Ok(mut file) = File::open(flac_path) {

        let mut text = String::new();
        file.read_to_string(&mut text).expect("Failed to read flac transcript.");

        // Remove linebreaks
        text = text.to_lowercase().replace(|c| {
            match c {
                '\n' | '\r' | '\t' => true,
                _ => false
            }

        }, " ");

        // Split up into unique words
        let mut parts: Vec<String> = text.split(' ').filter(|s| {
            !s.trim().is_empty()

        }).map(|s| {
            s.to_string()

        }).collect();

        parts.dedup();

        Some(parts.join(" "))

    } else {
        None
    }

}
*/
