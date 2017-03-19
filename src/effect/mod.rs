// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;
use clock_ticks;
use rand::{thread_rng, Rng};
use hyper::Client;
use hyper::header::Connection;
use edit_distance::edit_distance;


// Modules --------------------------------------------------------------------
mod effect;
mod stats;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::server::ServerConfig;
use ::db::models::{Effect as EffectModel, NewEffect as NewEffectModel};
//use ::db::schema::effects::dsl::{server_id, name as effect_name};
use ::db::schema::effects::table as effectTable;
use self::stats::EffectStatCache;

pub use self::stats::EffectStat;
pub use self::effect::Effect as Effect;


// Effects Registration -------------------------------------------------------
#[derive(Debug)]
pub struct EffectRegistry {
    effects: HashMap<String, Effect>,
    last_played: HashMap<String, u64>,
    stat_cache: EffectStatCache
}


// Public Interface -----------------------------------------------------------
impl EffectRegistry {

    pub fn new() -> EffectRegistry {
        EffectRegistry {
            effects: HashMap::new(),
            last_played: HashMap::new(),
            stat_cache: EffectStatCache::new()
        }
    }

    pub fn reload(&mut self, config: &ServerConfig) {

        //use ::db::schema::effects::dsl::server_id;

        self.effects.clear();

        //for effect in effectTable.filter(server_id.eq(&self.table_id))
        //          .load::<EffectModel>(&self.connection)
        //          .unwrap_or_else(|_| vec![]) {


        //}

        self.load_effects(config)
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.effects.contains_key(effect_name)
    }

    pub fn get_effect(&self, effect_name: &str) -> Option<&Effect> {
        self.effects.get(effect_name)
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

    pub fn rename_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect,
        effect_name: &str

    ) -> Result<(), String> {

        let mut new_effect_path = config.effects_path.clone();
        let mut new_transcript_path = config.effects_path.clone();

        if let Some(uploader) = effect.uploader() {
            new_effect_path.push(format!(
                "{}.{}.flac",
                effect_name,
                uploader.replace("#", "_")
            ))

        } else {
            new_effect_path.push(effect_name);
            new_effect_path.set_extension("flac");
        }

        new_transcript_path.push(effect_name);
        new_transcript_path.set_extension("txt");

        fs::rename(effect.to_path_str(), new_effect_path).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            // TODO rename effect in DB
            fs::rename(effect.transcript_path(), new_transcript_path).ok();
            self.reload(config);
            Ok(())
        })

    }

    pub fn delete_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect

    ) -> Result<(), String> {
        // TODO remove effect to DB
        fs::remove_file(effect.to_path_str()).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            fs::remove_file(effect.transcript_path()).ok();
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
        // TODO add effect to DB and get stats
        download_file(
            config.effects_path.clone(),
            effect_name,
            upload_url,
            Some(uploader),
            "flac"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            Ok(self.reload(config))
        })
    }

    pub fn download_transcript(
        &mut self,
        config: &ServerConfig,
        effect_name: &str,
        upload_url: &str

    ) -> Result<(), String> {
        // TODO update transcript for effect in DB
        download_file(
            config.effects_path.clone(),
            effect_name,
            upload_url,
            None,
            "txt"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            Ok(self.reload(config))
        })
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

    fn load_effects(&mut self, config: &ServerConfig) {

        // TODO load from database

        let start = clock_ticks::precise_time_ms();
        filter_dir(&config.effects_path, "flac", |name, path| {

            // let duration = get_flac_duration(path.clone()).unwrap_or(0);
            let descriptor: Vec<&str> = name.split('.').collect();
            let effect = if descriptor.len() == 2 {
                Effect::new(
                    descriptor[0],
                    path.clone(),
                    self.stat_cache.get(config, path, descriptor[0]),
                    Some(descriptor[1].replace("_", "#"))
                )

            } else {
                Effect::new(
                    descriptor[0],
                    path.clone(),
                    self.stat_cache.get(config, path, descriptor[0]),
                    None
                )
            };

            self.effects.insert(
                effect.name.clone(),
                effect.with_transcript()
            );

        });

        info!(
            "{} Effects loaded in {}ms.",
            self,
            clock_ticks::precise_time_ms() - start
        );

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
        if let Some(transcript) = effect.transcript() {
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
                                    stem.to_str().unwrap_or("").to_string(),
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
    nickname: Option<&str>,
    ext: &str

) -> Result<(), String> {

    if let Some(nickname) = nickname {
        directory.push(&format!("{}.{}.{}", effect_name, nickname, ext));

    } else {
        directory.push(&format!("{}.{}", effect_name, ext));
    }

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

