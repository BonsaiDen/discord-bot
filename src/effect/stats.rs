// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::cmp;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::collections::{HashMap, BTreeMap};


// External Dependencies ------------------------------------------------------
use toml;
use flac::{ReadStream, StreamReader, StreamIter};


// Internal Dependencies ------------------------------------------------------
use ::server::ServerConfig;


// Effect Statistics ----------------------------------------------------------
#[derive(Debug, Clone)]
pub struct EffectStat {
    pub duration_ms: u64,
    pub peak_db: f32,
    pub silent_start_samples: u64,
    pub silent_end_samples: u64
}


// Effect Statistics Cache ----------------------------------------------------
#[derive(Debug)]
pub struct EffectStatCache {
    cache: Option<HashMap<String, EffectStat>>
}


// Public Interface -----------------------------------------------------------
impl EffectStatCache {

    pub fn new() -> EffectStatCache {
        EffectStatCache {
            cache: None
        }
    }

    pub fn get(
        &mut self,
        config: &ServerConfig,
        path: PathBuf,
        effect_name: &str

    ) -> Option<EffectStat> {

        // TODO Dry / clean up
        self.load_cache(config);

        let cache_name = format!("{}", self);
        let (stat, modified) = if let Some(ref mut cache) = self.cache {

            let mut loaded = false;
            if !cache.contains_key(effect_name) {
                if let Ok(stat) = anaylze_flac(path) {
                    info!("{} Analyzing flac effect \"{}\"...", cache_name, effect_name);
                    cache.insert(effect_name.to_string(), stat);
                    loaded = true;
                }
            }

            (cache.get(effect_name).map(|stat| stat.clone()), loaded)

        } else {
            (None, false)
        };

        if modified {
            self.store_cache(config);
        }

        stat

    }

}


// Internal Interface ---------------------------------------------------------
impl EffectStatCache {

    fn load_cache(&mut self, config: &ServerConfig) {

        if self.cache.is_none() {

            let mut cache_path = config.effects_path.clone();
            cache_path.push("cache");
            cache_path.set_extension("toml");

            let mut cache = HashMap::new();
            if let Ok(table) = decode_toml_cache(cache_path) {
                for (effect_name, value) in table {
                    if let toml::Value::Table(ref table) = value {
                        cache.insert(effect_name.clone(), EffectStat {
                            duration_ms: table.get("duration_ms").map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0) as u64,
                            peak_db: table.get("peak_db").map(|v| v.as_float().unwrap_or(0.0)).unwrap_or(0.0) as f32,
                            silent_start_samples: table.get("silent_start_samples").map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0) as u64,
                            silent_end_samples: table.get("silent_end_samples").map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0) as u64
                        });
                    }
                }
            }

            self.cache = Some(cache);

        }

    }

    fn store_cache(&mut self, config: &ServerConfig) {

        let mut cache_path = config.effects_path.clone();
        cache_path.push("cache");
        cache_path.set_extension("toml");

        let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();
        if let Some(ref cache) = self.cache {
            for (key, value) in cache {
                let mut stats: BTreeMap<String, toml::Value> = BTreeMap::new();
                stats.insert("duration_ms".to_string(), toml::Value::Integer(value.duration_ms as i64));
                stats.insert("peak_db".to_string(), toml::Value::Float(value.peak_db as f64));
                stats.insert("silent_start_samples".to_string(), toml::Value::Integer(value.silent_start_samples as i64));
                stats.insert("silent_end_samples".to_string(), toml::Value::Integer(value.silent_end_samples as i64));
                toml.insert(key.clone(), toml::Value::Table(stats));
            }
        }

        encode_toml_cache(cache_path, toml).ok();

    }

}

// Traits ---------------------------------------------------------------------
impl fmt::Display for EffectStatCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cache) = self.cache {
            write!(f, "[EffectStatCache {} effect(s)]", cache.len())

        } else {
            write!(f, "[EffectStatCache (Empty)]")
        }
    }
}


// Helpers --------------------------------------------------------------------
fn decode_toml_cache(cache_path: PathBuf) -> Result<BTreeMap<String, toml::Value>, String> {
    File::open(cache_path)
        .map_err(|err| err.to_string())
        .and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .map_err(|err| err.to_string())
                .map(|_| buffer)
        })
        .and_then(|buffer| {
            toml::Parser::new(&buffer)
                .parse()
                .map_or_else(|| {
                    Err("Failed to parse cache toml.".to_string())

                }, |table| Ok(table))
        })
}

fn encode_toml_cache(cache_path: PathBuf, value: BTreeMap<String, toml::Value>) -> Result<(), String> {
    File::create(cache_path)
        .map_err(|err| err.to_string())
        .and_then(|mut file| {
            write!(file, "{}", toml::Value::Table(value))
                .map_err(|err| err.to_string())
        })

}

fn anaylze_flac(flac_path: PathBuf) -> Result<EffectStat, String> {
    StreamReader::<File>::from_file(flac_path.to_str().unwrap_or(""))
        .map_err(|_| "Failed to open flac file.".to_string())
        .and_then(|stream| {
            Ok(anaylze_flac_stream(stream))
        })
}

fn anaylze_flac_stream(stream: StreamReader<File>) -> EffectStat {

    let stream_info = stream.info();
    let samples: StreamIter<ReadStream<File>, i64> = StreamIter::new(stream);

    let mut sample_count = 0;
    let sum_squares = samples.into_iter().fold(0.0f64, |acc, s| {
        let sample = (s as f64 / 32768.0);
        if sample > 0.01 {
            sample_count += 1;
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
        silent_end_samples: 0
    }

}

