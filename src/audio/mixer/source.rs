// STD Dependencies -----------------------------------------------------------
use std::fs::File;


// External Dependencies ------------------------------------------------------
use flac;


// Internal Dependencies ------------------------------------------------------
use ::action::ActionOption;
use ::effect::Effect;


// Mixer Source Implementation ------------------------------------------------
pub struct MixerSource {
    active: bool,
    channels: usize,
    gain: f32,
    bitrate: i16,
    effect: Option<Effect>,
    action: ActionOption,
    stream: flac::StreamIter<flac::ReadStream<File>, i64>,
}

impl MixerSource {

    pub fn new(
        effect: Effect,
        action: ActionOption

    ) -> Result<MixerSource, ()> {
        let filename = effect.to_path_str().to_string();
        if let Ok(stream) = flac::StreamReader::<File>::from_file(&filename) {
            info!("[Mixer] Multiplier: {}", effect.auto_adjust_gain());
            Ok(MixerSource {
                active: true,
                channels: stream.info().channels as usize,
                gain: effect.auto_adjust_gain(),
                bitrate: effect.bitrate(),
                effect: Some(effect),
                action: action,
                stream: flac::StreamIter::new(stream),
            })

        } else {
            Err(())
        }
    }

    pub fn into_effect(mut self) -> (Effect, ActionOption) {
        (self.effect.take().unwrap(), self.action.take())
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {

        let gain = self.gain;
        let (mut written, mut iter) = (0, &mut self.stream);

        if self.bitrate < 64 {
            let rate = 8 * (96 - self.bitrate) as i16;
            for s in iter.take(buffer.len()).map(|s| {
                (((s as f32) * gain) as i16 / rate) * rate

            }) {
                buffer[written] = s;
                written += 1;
            }

        } else {
            for s in iter.take(buffer.len()).map(|s| {
                ((s as f32) * gain) as i16

            }) {
                buffer[written] = s;
                written += 1;
            }
        }

        if written > 0 {
            Some(written)

        } else {
            self.active = false;
            None
        }

    }

}

