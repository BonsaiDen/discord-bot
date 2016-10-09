// STD Dependencies -----------------------------------------------------------
use std::fs::File;


// External Dependencies ------------------------------------------------------
use flac;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;


// Mixer Source Implementation ------------------------------------------------
pub struct MixerSource {
    active: bool,
    channels: usize,
    stream: flac::StreamIter<flac::ReadStream<File>, i64>
}

impl MixerSource {

    pub fn new(effect: Effect) -> Result<MixerSource, ()> {
        let file = effect.to_path_str();
        if let Ok(stream) = flac::StreamReader::<File>::from_file(file) {
            Ok(MixerSource {
                active: true,
                channels: stream.info().channels as usize,
                stream: flac::StreamIter::new(stream)
            })

        } else {
            Err(())
        }
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

