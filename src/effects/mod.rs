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
use ::core::server::ServerConfig;


// Effect Abstraction ---------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Effect {
    pub name: String,
    path: PathBuf,
    uploader: Option<String>
}

impl Effect {

    fn new(name: &str, path: PathBuf, uploader: Option<String>) -> Effect {
        Effect {
            name: name.to_string(),
            path: path,
            uploader: uploader
        }
    }

    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
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
    effects: HashMap<String, Vec<Effect>>,
    effects_last_playback: HashMap<String, u64>
}


// Public Interface -----------------------------------------------------------
impl EffectRegistry {

    pub fn new() -> EffectRegistry {
        EffectRegistry {
            effects: HashMap::new(),
            effects_last_playback: HashMap::new()
        }
    }

    pub fn reload(&mut self, config: &ServerConfig) {
        self.effects.clear();
        self.effects_last_playback.clear();
        self.load_effects(config)
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.effects.contains_key(effect_name)
    }

    pub fn get_effect(&self, effect_name: &str) -> Option<Effect> {
        if let Some(effects) = self.effects.get(effect_name) {
            effects.get(0).and_then(|e| Some(e.clone()))

        } else {
            None
        }
    }

    pub fn map_patterns(
        &self,
        patterns: &[String],
        aliases: Option<&HashMap<String, Vec<String>>>,
        match_all: bool,
        config: &BotConfig

    ) -> Vec<Effect> {

        let effects: Vec<Effect> = patterns.iter()
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

    ) -> Option<Vec<Effect>> {

        let now = clock_ticks::precise_time_ms();

        // Find matching effect names
        let mut matching_effects: Vec<&str> = self.effects.keys().map(|n| {
            n.as_str()

        }).filter(|name| {
            // TODO clean up
            if match_all {
                match_name_pattern(name, pattern, 0, now)

            } else {
                let last_played = *self.effects_last_playback.get(*name).unwrap();
                match_name_pattern(
                    name,
                    pattern,
                    last_played + config.effect_playback_separation_ms,
                    now
                )
            }

        }).collect();

        // Find matching alias names, if provided
        if let Some(aliases) = aliases {

            let matching_aliases: Vec<&str> = aliases.keys().map(|n| {
                n.as_str()

            }).filter(|name| match_name_pattern(name, pattern, 0, now)).collect();

            matching_effects.extend(matching_aliases);

        }

        if match_all {
            let mut effects = Vec::new();
            for m in matching_effects {
                if let Some(e) = self.effects.get(m) {
                    effects.append(&mut e.clone())
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
        config: &BotConfig

    ) -> Option<Vec<Effect>> {

        // Select one random effect...
        if let Some(name) = thread_rng().choose(&effects[..]) {

            // ...selected effect is already a full effect
            if let Some(effect) = self.effects.get(*name) {
                Some(effect.clone())

            // ...selected effect is an alias, so we need to resolve its mapped effect
            } else if let Some(aliases) = aliases {
                if let Some(aliased_effects) = aliases.get(*name) {
                    Some(self.map_patterns(aliased_effects, None, false, config))

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

            let description: Vec<&str> = name.split('.').collect();
            let effect = match *description.as_slice() {
                [name, uploader] => {
                    Effect::new(name, path, Some(uploader.replace("_", "#")))
                },
                [name] => {
                    Effect::new(name, path, None)
                },
                _ => unreachable!()
            };

            self.effects_last_playback.insert(effect.name.clone(), 0);
            self.effects.insert(effect.name.clone(), vec![effect]);

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
fn match_name_pattern(
    name: &str,
    pattern: &str,
    last_played: u64,
    now: u64

) -> bool {

    let len = pattern.len();

    // Random
    if pattern == "*" {
        // Filter out recently played effects
        last_played < now

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
        } else if name.starts_with(&format!("{}_", pattern)) {
            // Filter out recently played effects
            last_played < now

        } else {
            false
        }

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

