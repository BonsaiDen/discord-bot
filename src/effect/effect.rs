// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
use ::effect::EffectStat;


// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;


// Effect Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Effect {
    pub name: String,
    server_id: ServerId,
    path: PathBuf,
    stats: Option<EffectStat>,
    uploader: Option<String>,
    transcript: Option<String>,
    bitrate: Option<u64>,
}

impl Effect {

    pub fn new(
        server_id: ServerId,
        name: &str,
        path: PathBuf,
        stats: Option<EffectStat>,
        uploader: Option<String>

    ) -> Effect {
        Effect {
            name: name.to_string(),
            server_id: server_id,
            path: path,
            stats: stats,
            uploader: uploader,
            transcript: None,
            bitrate: None
        }
    }

    pub fn sync_to_db(&self, connection: &SqliteConnection) {

        use ::db::models::NewEffect;
        use ::db::schema::effects;

        let sid = format!("{}", self.server_id);

        let uploader = self.uploader.as_ref().unwrap_or(&String::new()).clone();
        let transcript = self.transcript.as_ref().unwrap_or(&String::new()).clone();

        let new_effect = NewEffect {
            server_id: &sid,
            name: &self.name,
            uploader: &uploader,
            transcript: &transcript,
            peak_db: self.stats.as_ref().unwrap().peak_db,
            duration_ms: self.stats.as_ref().unwrap().duration_ms as i32,
            silent_start_samples: self.stats.as_ref().unwrap().silent_start_samples as i32,
            silent_end_samples: self.stats.as_ref().unwrap().silent_end_samples as i32
        };

        diesel::insert(&new_effect).into(effects::table)
               .execute(connection)
               .expect("Error saving new effect.");

    }

    pub fn auto_adjust_gain(&self) -> f32 {
        if let Some(stats) = self.stats.as_ref() {
            let db_gain_diff = -26.0 - (stats.peak_db);
            let gain = 10.0f32.powf(db_gain_diff / 20.0) - 1.0;
            1.0 + gain * 0.75

        } else {
            1.0
        }
    }

    pub fn with_transcript(mut self) -> Effect {
        self.transcript = load_transcript(self.transcript_path());
        self
    }

    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn transcript_path(&self) -> PathBuf {
        if self.uploader.is_some() {
            self.path.with_extension("").with_extension("txt")

        } else {
            self.path.with_extension("txt")
        }
    }

    pub fn uploader(&self) -> Option<&String> {
        self.uploader.as_ref()
    }

    pub fn transcript(&self) -> Option<&String> {
        self.transcript.as_ref()
    }

    pub fn bitrate(&self) -> i16 {
        self.bitrate.unwrap() as i16
    }

    pub fn clone_with_bitrate(&self, bitrate: u64) -> Self {
        Effect {
            server_id: self.server_id,
            name: self.name.to_string(),
            path: self.path.clone(),
            stats: self.stats.clone(),
            uploader: self.uploader.clone(),
            transcript: None,
            bitrate: Some(bitrate)
        }
    }

}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            server_id: self.server_id,
            name: self.name.to_string(),
            path: self.path.clone(),
            stats: self.stats.clone(),
            uploader: self.uploader.clone(),
            transcript: None,
            bitrate: self.bitrate
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


// Helpers --------------------------------------------------------------------
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

