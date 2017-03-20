// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::path::PathBuf;


// Internal Dependencies ------------------------------------------------------
use ::effect::EffectStat;


// Effect Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Effect {
    pub name: String,
    path: PathBuf,
    stats: EffectStat,
    uploader: String,
    transcript: String,
    bitrate: Option<u64>
}

impl Effect {

    pub fn new(
        name: &str,
        path: PathBuf,
        stats: EffectStat,
        uploader: String,
        transcript: String

    ) -> Effect {
        Effect {
            name: name.to_string(),
            path: path,
            stats: stats,
            uploader: uploader,
            transcript: transcript,
            bitrate: None
        }
    }

    pub fn auto_adjust_gain(&self) -> f32 {
        let db_gain_diff = -26.0 - (self.stats.peak_db);
        let gain = 10.0f32.powf(db_gain_diff / 20.0) - 1.0;
        1.0 + gain * 0.75
    }

    pub fn to_path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn uploader(&self) -> Option<&str> {
        if self.uploader.is_empty() {
            None

        } else {
            Some(&self.uploader)
        }
    }

    pub fn transcript(&self) -> &str {
        &self.transcript
    }

    pub fn bitrate(&self) -> i16 {
        self.bitrate.unwrap() as i16
    }

    pub fn clone_with_bitrate(&self, bitrate: u64) -> Self {
        Effect {
            name: self.name.to_string(),
            path: self.path.clone(),
            stats: self.stats.clone(),
            uploader: self.uploader.clone(),
            transcript: "".to_string(),
            bitrate: Some(bitrate)
        }
    }

}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            name: self.name.to_string(),
            path: self.path.clone(),
            stats: self.stats.clone(),
            uploader: self.uploader.clone(),
            transcript: "".to_string(),
            bitrate: self.bitrate
        }
    }
}


// Traits ---------------------------------------------------------------------
impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(uploader) = self.uploader() {
            write!(f, "[Effect {} by {}]", self.name, uploader)

        } else {
            write!(f, "[Effect {}]", self.name)
        }
    }
}

