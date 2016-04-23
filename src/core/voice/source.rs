// Internal Dependencies ------------------------------------------------------
use super::super::Effect;


// Source List Implementation -------------------------------------------------
pub struct SourceList {
    effects: Vec<Effect>,
    source: Option<Source>,
    delay: usize
}

impl SourceList {

    fn new(effects: Vec<Effect>, delay: usize) -> SourceList {

        let mut list = SourceList {
            effects: effects,
            source: None,
            delay: delay / 20
        };

        list.update_source();
        list

    }

    fn get_active_source(&mut self) -> Option<&mut Source> {
        self.source.as_mut()
    }

    fn update_source(&mut self) {

        // Check if there is either no current source or the current source has ended
        if self.source.is_none() || !self.source.as_ref().unwrap().is_active() {

            // Check if there is another, valid file which should be played
            while let Some(effect) = self.effects.pop() {

                // Use the first valid file as the next source
                if let Ok(source) = Source::new(effect, self.delay) {
                    self.delay = 0; // Only delay the first source
                    self.source = Some(source);
                    break;
                }

            }
        }

    }

    fn is_active(&self) -> bool {
        !self.source.is_none() && self.source.as_ref().unwrap().is_active()
    }

}


// Source Implementation ------------------------------------------------------
pub struct Source {
    active: bool,
    channels: usize,
    delay: usize
}

impl Source {

    pub fn new(effect: Effect, delay: usize) -> Result<Source, ()> {
        Ok(Source {
            active: true,
            channels: 1,
            delay: delay
        })
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        None
    }

}

