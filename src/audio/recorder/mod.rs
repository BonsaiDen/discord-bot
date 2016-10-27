// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use chrono;
use clock_ticks;


// Modules --------------------------------------------------------------------
mod track;
use self::track::Track;


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;
use discord::voice::AudioReceiver;


// Audio Recorder Abstraction -------------------------------------------------
pub struct Recorder {
    tracks: HashMap<u32, Track>,
    recording_path: PathBuf,
    chunk_duration: u32,
    start_timestamp: u64
}


// Public Interface -----------------------------------------------------------
impl Recorder {

    pub fn new(mut recording_path: PathBuf, chunk_duration: u32) -> Recorder {

        recording_path.push(format!("{}", chrono::Local::now()));

        Recorder {
            tracks: HashMap::new(),
            recording_path: recording_path,
            chunk_duration: chunk_duration,
            start_timestamp: clock_ticks::precise_time_ms()
        }

    }

}

// Internal Interface ---------------------------------------------------------
impl Recorder {

    fn get_track(&mut self, source_id: u32) -> &mut Track {

        let chunk_duration = self.chunk_duration;
        let start_timestamp = self.start_timestamp;
        let path = self.recording_path.clone();

        self.tracks.entry(source_id).or_insert_with(|| {
            Track::new(
                source_id,
                chunk_duration,
                clock_ticks::precise_time_ms() - start_timestamp,
                path
            )
        })

    }

    fn flush(&mut self) {
        for track in self.tracks.values_mut() {
            track.flush();
        }
    }

}


// Traits ---------------------------------------------------------------------
impl Drop for Recorder {
    fn drop(&mut self) {
        self.flush();
    }
}


// Recorder Receiver Implementation -------------------------------------------
impl AudioReceiver for Recorder {

    fn speaking_update(&mut self, source_id: u32, user_id: &UserId, _: bool) {
        self.get_track(source_id).set_user_id(user_id);
    }

    fn voice_packet(
        &mut self,
        source_id: u32,
        sequence: u16,
        timestamp: u32,
        stereo: bool,
        data: &[i16]
    ) {
        self.get_track(source_id).add_voice_packet(
            sequence,
            timestamp / 48,
            clock_ticks::precise_time_ms(),
            if stereo {
                2
            } else {
                1
            },
            data
        );
    }

}

