// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use clock_ticks;
use rand::{thread_rng, Rng};
use hyper::Client;
use hyper::header::Connection;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::ServerConfig;


// Effect Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Effect {
    pub name: String,
    path: PathBuf,
    last_played: u64,
    uploader: Option<String>,
    transcript: Option<Vec<String>>
}

impl Effect {

    fn new(
        name: &str,
        path: PathBuf,
        uploader: Option<String>,
        transcript: Option<Vec<String>>

    ) -> Effect {
        Effect {
            name: name.to_string(),
            path: path,
            last_played: 0,
            uploader: uploader,
            // TODO add commands to show and set effect transcriptions
            transcript: transcript
        }
    }

    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            name: self.name.to_string(),
            path: self.path.clone(),
            last_played: self.last_played,
            uploader: self.uploader.clone(),
            transcript: None
        }
    }
}


// Traits ---------------------------------------------------------------------
impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref uploader) = self.uploader {
            write!(f, "[Effect {} by {}]", self.name, uploader)

        } else {
            write!(f, "[Effect {}]", self.name)
        }
    }
}


// Effects Registration -------------------------------------------------------
#[derive(Debug)]
pub struct EffectRegistry {
    effects: HashMap<String, Effect>
}


// Public Interface -----------------------------------------------------------
impl EffectRegistry {

    pub fn new() -> EffectRegistry {
        EffectRegistry {
            effects: HashMap::new()
        }
    }

    pub fn reload(&mut self, config: &ServerConfig) {
        self.effects.clear();
        self.load_effects(config)
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.effects.contains_key(effect_name)
    }

    pub fn get_effect(&self, effect_name: &str) -> Option<&Effect> {
        self.effects.get(effect_name)
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
             .filter(|e| e.is_some())
             .map(|e| e.unwrap())
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

    pub fn rename_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect,
        effect_name: &str

    ) -> Result<(), String> {

        let mut new_path = config.effects_path.clone();

        if let Some(ref uploader) = effect.uploader {
            new_path.push(format!(
                "{}.{}.flac",
                effect_name,
                uploader.replace("#", "_")
            ))

        } else {
            new_path.push(effect_name);
            new_path.set_extension("flac");
        }

        fs::rename(effect.path.clone(), new_path).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            self.reload(config);
            Ok(())
        })

    }

    pub fn delete_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect

    ) -> Result<(), String> {
        fs::remove_file(effect.path.clone()).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            self.reload(config);
            Ok(())
        })
    }

    pub fn download_effect(
        &mut self,
        config: &ServerConfig,
        effect_name: &str,
        upload_url: &str,
        uploader: &str

    ) -> Result<(), String> {
        download_file(
            config.effects_path.clone(),
            effect_name,
            upload_url,
            uploader,
            "flac"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            Ok(self.reload(config))
        })
    }

}


// Internal Inteface ----------------------------------------------------------
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
                pattern,
                !match_all,
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

    fn load_effects(&mut self, config: &ServerConfig) {

        filter_dir(&config.effects_path, "flac", |name, path| {

            // Try to load a transcript if present
            let mut transcript_path = path.clone();
            transcript_path.set_extension("txt");

            let transcript = if let Ok(mut file) = File::open(transcript_path) {

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

                Some(parts)

            } else {
                None
            };

            // Create effect from flac
            let descriptor: Vec<&str> = name.split('.').collect();
            let effect = match *descriptor.as_slice() {
                [name, uploader] => {
                    Effect::new(
                        name,
                        path,
                        Some(uploader.replace("_", "#")),
                        transcript
                    )
                },
                [name] => {
                    Effect::new(
                        name,
                        path,
                        None,
                        transcript
                    )
                },
                _ => unreachable!()
            };

            self.effects.insert(effect.name.clone(), effect);

        });

        info!("{} Effects loaded.", self);

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
    pattern: &str,
    ignore_recent: bool,
    recent_threshold: u64

) -> bool {

    let len = pattern.len();

    // Name: Random
    if pattern == "*" {
        ignore_recent || !was_recently_played(effect, recent_threshold)

    // Name: Contains
    } else if len > 2 && pattern.starts_with('*') && pattern.ends_with('*') {
        effect.name.contains(&pattern[1..len - 1])

    // Transcript: Contains
    } else if len > 2 && pattern.starts_with('"') && pattern.ends_with('"') {
        if let Some(ref transcript) = effect.transcript {
            transcript.contains(&pattern[1..len - 1].to_string())

        } else {
            false
        }

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
            ignore_recent || !was_recently_played(effect, recent_threshold)

        } else {
            false
        }

    } else {
        false
    }

}

fn was_recently_played(effect: &Effect, threshold: u64) -> bool {
    effect.last_played + threshold < clock_ticks::precise_time_ms()
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

fn filter_dir<F: FnMut(String, PathBuf)>(
    path: &PathBuf,
    ext: &str,
    mut callback: F
) {
    if let Ok(listing) = fs::read_dir(path) {
        for entry in listing {
            if let Ok(entry) = entry {
                if entry.file_type().unwrap().is_file() {
                    let path = entry.path();
                    if path.extension().unwrap_or_else(|| OsStr::new("")) == ext {
                        if let Some(stem) = path.file_stem() {
                            if stem != "" {
                                callback(
                                    stem.to_str().unwrap().to_string(),
                                    PathBuf::from(path.clone())
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

fn download_file(
    mut directory: PathBuf,
    effect_name: &str,
    url: &str,
    nickname: &str,
    ext: &str

) -> Result<(), String> {

    directory.push(&format!("{}.{}.{}", effect_name, nickname, ext));

    let client = Client::new();
    client.get(url)
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut resp| {
            let mut buffer = Vec::<u8>::new();
            resp.read_to_end(&mut buffer)
                .map_err(|err| err.to_string())
                .map(|_| buffer)
        })
        .and_then(|buffer| {
            File::create(directory)
                .map_err(|err| err.to_string())
                .and_then(|mut file| {
                    file.write_all(&buffer)
                        .map_err(|err| err.to_string())
                })
        })

}

