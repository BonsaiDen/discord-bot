// STD Dependencies -----------------------------------------------------------
use std::fs::File;


// External Dependencies ------------------------------------------------------
use flac;


// Internal Dependencies ------------------------------------------------------
use super::super::Effect;


// Statics --------------------------------------------------------------------
static MILLIS_PER_SOURCE_TICK: usize = 20;


// Source List Implementation -------------------------------------------------
pub struct SourceList {
    effects: Vec<Effect>,
    source: Option<Source>,
    delay_ticks: usize
}

impl SourceList {

    pub fn new(effects: Vec<Effect>, delay: usize) -> SourceList {

        let mut list = SourceList {
            effects: effects,
            source: None,
            delay_ticks: delay / MILLIS_PER_SOURCE_TICK
        };

        list.effects.reverse();
        list.update();
        list

    }

    pub fn get_active_source(&mut self) -> Option<&mut Source> {
        self.source.as_mut()
    }

    pub fn update(&mut self) {

        // Check if there is either no current source or the current source has ended
        if self.source.is_none() || !self.source.as_ref().unwrap().is_active() {

            // Check if there is another, valid file which should be played
            while let Some(effect) = self.effects.pop() {

                // Use the first valid file as the next source
                if let Ok(source) = Source::new(effect, self.delay_ticks) {
                    self.delay_ticks = 0; // Only delay the first source
                    self.source = Some(source);
                    break;
                }

            }
        }

    }

    pub fn is_active(&self) -> bool {
        !self.source.is_none() && self.source.as_ref().unwrap().is_active()
    }

}


// Source Implementation ------------------------------------------------------
pub struct Source {
    active: bool,
    channels: usize,
    stream: flac::StreamIter<flac::ReadStream<File>, i64>,
    delay_ticks: usize
}

impl Source {

    pub fn new(effect: Effect, delay_ticks: usize) -> Result<Source, ()> {
        let file = effect.to_path_str();
        if let Ok(stream) = flac::StreamReader::<File>::from_file(&file) {
            Ok(Source {
                active: true,
                channels: stream.info().channels as usize,
                stream: flac::StreamIter::new(stream),
                delay_ticks: delay_ticks
            })

        } else {
            Err(())
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {

        if self.delay_ticks > 0 {
            self.delay_ticks -= 1;
            Some(0)

        } else {
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

}

