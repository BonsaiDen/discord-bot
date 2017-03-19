// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
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
use flac::{ReadStream, StreamReader, StreamIter};


// Modules --------------------------------------------------------------------
mod effect;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::server::ServerConfig;
use ::db::models::{Effect as EffectModel, NewEffect as NewEffectModel};
use ::db::schema::effects::dsl::{server_id, name as effect_name};
use ::db::schema::effects::table as effectTable;

pub use self::effect::Effect as Effect;


// Effect Statistics ----------------------------------------------------------
#[derive(Debug, Clone)]
pub struct EffectStat {
    pub duration_ms: u64,
    pub peak_db: f32,
    pub silent_start_samples: u64,
    pub silent_end_samples: u64
}


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
        self.effects.clear();
        self.load_effects(config)
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

    pub fn rename_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect,
        name: &str

    ) -> Result<(), String> {

        let mut new_effect_path = config.effects_path.clone();
        if let Some(uploader) = effect.uploader() {
            new_effect_path.push(format!(
                "{}.{}.flac",
                name,
                uploader.replace("#", "_")
            ))

        } else {
            new_effect_path.push(name);
            new_effect_path.set_extension("flac");
        }

        // TODO use a transaction?
        let q = effectTable.filter(server_id.eq(&config.table_id)).filter(effect_name.eq(effect.name.clone()));
        if diesel::update(q).set(effect_name.eq(name)).execute(&config.connection).is_ok() {
            fs::rename(effect.to_path_str(), new_effect_path).map_err(|err| {
                err.to_string()

            }).and_then(|_| {
                Ok(self.reload(config))
            })

        } else {
            Err("Failed to rename effect in database.".to_string())
        }

    }

    pub fn delete_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect

    ) -> Result<(), String> {
        // TODO use a transaction?
        let q = effectTable.filter(server_id.eq(&config.table_id)).filter(effect_name.eq(&effect.name));
        if diesel::delete(q).execute(&config.connection).is_ok() {
            fs::remove_file(effect.to_path_str()).map_err(|err| {
                err.to_string()

            }).and_then(|_| {
                Ok(self.reload(config))
            })

        } else {
            Err("Failed to delete effect from database.".to_string())
        }
    }

    pub fn download_effect(
        &mut self,
        config: &ServerConfig,
        name: &str,
        upload_url: &str,
        uploader: &str

    ) -> Result<(), String> {

        download_file(
            config.effects_path.clone(),
            name,
            upload_url,
            Some(uploader),
            "flac"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|effect_path| {

            // TODO dry error handling
            if let Ok(stats) = analyze_flac(&effect_path) {

                let new_effect = NewEffectModel {
                    server_id: &config.table_id,
                    name: name,
                    uploader: uploader,
                    peak_db: stats.peak_db,
                    duration_ms: stats.duration_ms as i32,
                    silent_start_samples: stats.silent_start_samples as i32,
                    silent_end_samples: stats.silent_end_samples as i32,
                    transcript: ""
                };

                if diesel::insert(&new_effect).into(effectTable).execute(&config.connection).is_ok() {
                    Ok(self.reload(config))

                } else {
                    fs::remove_file(effect_path).map_err(|err| {
                        err.to_string()

                    }).and_then(|_| {
                        Err("Failed to analyze uploaded flac file.".to_string())
                    })
                }

            } else {
                fs::remove_file(effect_path).map_err(|err| {
                    err.to_string()

                }).and_then(|_| {
                    Err("Failed to analyze uploaded flac file.".to_string())
                })
            }

        })

    }

    pub fn download_transcript(
        &mut self,
        _: &ServerConfig,
        _: &str,
        _: &str

    ) -> Result<(), String> {
        // TODO update transcript for effect in DB
        // TODO download text instead of file
        /*
        download_file(
            config.effects_path.clone(),
            name,
            upload_url,
            None,
            "txt"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            Ok(self.reload(config))
        })
        */
        Ok(())
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

        let start = clock_ticks::precise_time_ms();
        for effect in effectTable.filter(server_id.eq(&config.table_id))
                  .load::<EffectModel>(&config.connection)
                  .unwrap_or_else(|_| vec![]) {

            let effect = self.effect_from_model(config, effect);
            self.effects.insert(effect.name.clone(), effect);
        }

        info!(
            "{} Effects loaded in {}ms.",
            self,
            clock_ticks::precise_time_ms() - start
        );

    }

    fn effect_from_model(
        &self,
        config: &ServerConfig,
        effect: EffectModel

    ) -> Effect {

        let mut path = PathBuf::new();
        path.push(config.effects_path.clone());

        if effect.uploader.is_empty() {
            path.push(effect.name.clone());

        } else {
            path.push(format!("{}.{}.", effect.name, effect.uploader));
        }

        path.set_extension("flac");

        let e = Effect::new(
            effect.name.as_str(),
            path,
            Some(EffectStat {
                duration_ms: effect.duration_ms as u64,
                peak_db: effect.peak_db,
                silent_start_samples: effect.silent_start_samples as u64,
                silent_end_samples: effect.silent_end_samples as u64
            }),
            Some(effect.uploader)
        );

        e.with_transcript(effect.transcript)

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

fn download_file(
    mut directory: PathBuf,
    name: &str,
    url: &str,
    nickname: Option<&str>,
    ext: &str

) -> Result<PathBuf, String> {

    if let Some(nickname) = nickname {
        directory.push(&format!("{}.{}.{}", name, nickname, ext));

    } else {
        directory.push(&format!("{}.{}", name, ext));
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
            File::create(directory.clone())
                .map_err(|err| err.to_string())
                .and_then(|mut file| {
                    file.write_all(&buffer)
                        .map_err(|err| err.to_string())
                        .and_then(|_| Ok(directory))
                })
        })

}

fn analyze_flac(flac_path: &PathBuf) -> Result<EffectStat, String> {
    StreamReader::<File>::from_file(flac_path.to_str().unwrap_or(""))
        .map_err(|_| "Failed to open flac file.".to_string())
        .and_then(|stream| {
            Ok(analyze_flac_stream(stream))
        })
}

fn analyze_flac_stream(stream: StreamReader<File>) -> EffectStat {

    let stream_info = stream.info();
    let samples: StreamIter<ReadStream<File>, i64> = StreamIter::new(stream);

    let mut sample_count = 0;
    let mut last_active_sample = 0;

    let sum_squares = samples.into_iter().fold(0.0f64, |acc, s| {
        let sample = s as f64 / 32768.0;
        if sample > 0.01 {
            sample_count += 1;
            if sample > 0.025 {
                last_active_sample = sample_count;
            }
            acc + sample.powf(2.0f64)

        } else {
            acc
        }
    });

    let rms = (sum_squares / (sample_count as f64)).sqrt();
    EffectStat {
        duration_ms: (stream_info.total_samples * 1000) / stream_info.sample_rate as u64,
        peak_db: (20.0 * rms.log(10.0)) as f32,
        silent_start_samples: 0,
        silent_end_samples: sample_count - last_active_sample
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
