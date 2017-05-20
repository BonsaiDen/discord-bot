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
    effect: Option<Effect>,
    action: ActionOption,
    stream: flac::StreamIter<flac::ReadStream<File>, i64>,
}

impl MixerSource {

    pub fn new(
        effect: Effect,
        action: ActionOption

    ) -> Result<MixerSource, (Effect, ActionOption)> {
        let filename = effect.to_path_str().to_string();
        if let Ok(stream) = flac::StreamReader::<File>::from_file(&filename) {
            info!("[Mixer] Multiplier: {}", effect.auto_adjust_gain());
            Ok(MixerSource {
                active: true,
                channels: stream.info().channels as usize,
                gain: effect.auto_adjust_gain(),
                effect: Some(effect),
                action: action,
                stream: flac::StreamIter::new(stream),
            })

        } else {
            Err((effect, action))
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

    pub fn read_frame(&mut self, bitrate: u64, buffer: &mut [i16]) -> Option<usize> {

        let gain = self.gain;
        let (mut written, mut iter) = (0, &mut self.stream);

        // Downsampling
        // TODO reduce code required
        if bitrate <= 16 {

            buffer[0] = 0;

            let rc = 1.0 / (4000.0 * 2.0 * 3.14);
            let dt = 1.0 / 48000.0;
            let alpha = dt / (rc + dt);

            for s in iter.take(buffer.len()).map(|s| {
                (s as f32) * gain

            }) {
                let b = buffer[written] as f32 + (alpha * (s - buffer[written] as f32));
                let i = written / 12 * 12;
                buffer[i] = b as i16;
                buffer[i + 1] = b as i16;
                buffer[i + 2] = b as i16;
                buffer[i + 3] = b as i16;
                buffer[i + 4] = b as i16;
                buffer[i + 5] = b as i16;
                buffer[i + 6] = b as i16;
                buffer[i + 7] = b as i16;
                buffer[i + 8] = b as i16;
                buffer[i + 9] = b as i16;
                buffer[i + 10] = b as i16;
                buffer[i + 11] = b as i16;
                written += 1;
            }

        } else if bitrate <= 32 {

            buffer[0] = 0;

            let rc = 1.0 / (8000.0 * 2.0 * 3.14);
            let dt = 1.0 / 48000.0;
            let alpha = dt / (rc + dt);

            for s in iter.take(buffer.len()).map(|s| {
                (s as f32) * gain

            }) {
                let b = buffer[written] as f32 + (alpha * (s - buffer[written] as f32));
                let i = written / 6 * 6;
                buffer[i] = b as i16;
                buffer[i + 1] = b as i16;
                buffer[i + 2] = b as i16;
                buffer[i + 3] = b as i16;
                buffer[i + 4] = b as i16;
                buffer[i + 5] = b as i16;
                written += 1;
            }

        } else if bitrate <= 48 {

            buffer[0] = 0;

            let rc = 1.0 / (12000.0 * 2.0 * 3.14);
            let dt = 1.0 / 48000.0;
            let alpha = dt / (rc + dt);

            for s in iter.take(buffer.len()).map(|s| {
                (s as f32) * gain

            }) {
                let b = buffer[written] as f32 + (alpha * (s - buffer[written] as f32));
                let i = written / 4 * 4;
                buffer[i] = b as i16;
                buffer[i + 1] = b as i16;
                buffer[i + 2] = b as i16;
                buffer[i + 3] = b as i16;
                written += 1;
            }

        } else if bitrate <= 63 {

            buffer[0] = 0;

            let rc = 1.0 / (24000.0 * 2.0 * 3.14);
            let dt = 1.0 / 48000.0;
            let alpha = dt / (rc + dt);

            for s in iter.take(buffer.len()).map(|s| {
                (s as f32) * gain

            }) {
                let b = buffer[written] as f32 + (alpha * (s - buffer[written] as f32));
                let i = written / 2 * 2;
                buffer[i] = b as i16;
                buffer[i + 1] = b as i16;
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

