// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};


// External Dependencies ------------------------------------------------------
use clock_ticks;
use diesel;
use diesel::prelude::*;
use diesel::Connection as DieselConnection;
use hyper::Client;
use hyper::header::Connection;
use flac::{ReadStream, StreamReader, StreamIter};


// Internal Dependencies ------------------------------------------------------
use ::server::ServerConfig;
use ::db::schema::effects::table as effectTable;
use ::db::schema::effects::dsl::{server_id, name as effect_name};
use ::db::models::{Effect as EffectModel, NewEffect as NewEffectModel};
use ::effect::{EffectRegistry, Effect, EffectStat};


// Public Effect Management Interface -----------------------------------------
impl EffectRegistry {

    pub fn reload_effects(&mut self, config: &ServerConfig) {
        self.effects.clear();
        self.load_effects(config);
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

        let q = effectTable.filter(server_id.eq(&config.table_id)).filter(effect_name.eq(effect.name.clone()));
        config.connection.transaction::<_, Box<::std::error::Error>, _>(|| {
            try!(diesel::delete(q).execute(&config.connection));
            try!(fs::rename(effect.to_path_str(), new_effect_path));
            Ok(self.reload_effects(config))

        }).map_err(|_| {
           "Failed to rename effect from database.".to_string()
        })

    }

    pub fn delete_effect(
        &mut self,
        config: &ServerConfig,
        effect: &Effect

    ) -> Result<(), String> {
        let q = effectTable.filter(server_id.eq(&config.table_id)).filter(effect_name.eq(&effect.name));
        config.connection.transaction::<_, Box<::std::error::Error>, _>(|| {
            try!(diesel::delete(q).execute(&config.connection));
            try!(fs::remove_file(effect.to_path_str()));
            Ok(self.reload_effects(config))

        }).map_err(|_| {
           "Failed to delete effect from database.".to_string()
        })
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

        ).and_then(|effect_path| {
            analyze_flac(&effect_path).map_err(|err| {
                err.to_string()

            }).and_then(|stats| {
                diesel::insert(&NewEffectModel {
                    server_id: &config.table_id,
                    name: name,
                    uploader: uploader,
                    peak_db: stats.peak_db,
                    duration_ms: stats.duration_ms as i32,
                    silent_start_samples: stats.silent_start_samples as i32,
                    silent_end_samples: stats.silent_end_samples as i32,
                    transcript: ""

                }).into(effectTable)
                  .execute(&config.connection)
                  .and_then(|_| {

                    Ok(self.reload_effects(config))

                }).map_err(|_| {
                    "Failed to analyze uploaded flac file.".to_string()
                })

            }).map_err(|err| {
                fs::remove_file(effect_path).map_err(|err| {
                    err.to_string()

                }).ok();
                err
            })

        }).map_err(|err| {
            err.to_string()
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
            Ok(self.reload_effects(config))
        })
        */
        Ok(())
    }

}


// Internal Interface ---------------------------------------------------------
impl EffectRegistry {

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

        let name = if effect.uploader.is_empty() {
            effect.name.clone()

        } else {
            format!("{}.{}.", effect.name, effect.uploader.replace("#", "_"))
        };

        let mut path = PathBuf::new();
        path.push(config.effects_path.clone());
        path.push(name);
        path.set_extension("flac");

        Effect::new(
            effect.name.as_str(),
            path,
            EffectStat {
                duration_ms: effect.duration_ms as u64,
                peak_db: effect.peak_db,
                silent_start_samples: effect.silent_start_samples as u64,
                silent_end_samples: effect.silent_end_samples as u64
            },
            effect.uploader,
            effect.transcript
        )

    }

}


// Helpers --------------------------------------------------------------------
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
        let sample = f64::from(s) / 32768.0;
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
        duration_ms: (stream_info.total_samples * 1000) / u64::from(stream_info.sample_rate),
        peak_db: (20.0 * rms.log(10.0)) as f32,
        silent_start_samples: 0,
        silent_end_samples: sample_count - last_active_sample
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
        directory.push(&format!("{}.{}.{}", name, nickname.replace("#", "_"), ext));

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

