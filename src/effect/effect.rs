// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;


// Internal Dependencies ------------------------------------------------------
use ::effect::EffectStat;


// Effect Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Effect {
    pub name: String,
    path: PathBuf,
    stats: Option<EffectStat>,
    uploader: Option<String>,
    transcript: Option<String>
}

impl Effect {

    pub fn new(
        name: &str,
        path: PathBuf,
        stats: Option<EffectStat>,
        uploader: Option<String>

    ) -> Effect {
        Effect {
            name: name.to_string(),
            path: path,
            stats: stats,
            uploader: uploader,
            transcript: None
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

}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            name: self.name.to_string(),
            path: self.path.clone(),
            stats: self.stats.clone(),
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

