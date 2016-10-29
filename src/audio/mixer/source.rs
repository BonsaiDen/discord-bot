// STD Dependencies -----------------------------------------------------------
use std::fs::File;


// External Dependencies ------------------------------------------------------
use flac;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;


// Mixer Source Implementation ------------------------------------------------
pub struct MixerSource {
    active: bool,
    channels: usize,
    effect: Option<Effect>,
    playback_id: usize,
    stream: flac::StreamIter<flac::ReadStream<File>, i64>,
}

impl MixerSource {

    pub fn new(effect: Effect, playback_id: usize) -> Result<MixerSource, ()> {
        let filename = effect.to_path_str().to_string();
        if let Ok(stream) = flac::StreamReader::<File>::from_file(&filename) {
            Ok(MixerSource {
                active: true,
                channels: stream.info().channels as usize,
                effect: Some(effect),
                playback_id: playback_id,
                stream: flac::StreamIter::new(stream)
            })

        } else {
            Err(())
        }
    }

    pub fn into_effect(mut self) -> (Effect, usize) {
        (self.effect.take().unwrap(), self.playback_id)
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {

        let (mut written, mut iter) = (0, &mut self.stream);
        for s in iter.take(buffer.len()).map(|s| {
            s as i16

        }) {
            buffer[written] = s;
            written += 1;
        }

        if written > 0 {
            Some(written)

        } else {
            self.active = false;
            None
        }

    }

}

