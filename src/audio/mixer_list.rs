// Internal Dependencies ------------------------------------------------------
use super::MixerSource;
use ::effect::Effect;


// Mixer Source List Implementation -------------------------------------------
pub struct MixerList {
    effects: Vec<Effect>,
    source: Option<MixerSource>
}

impl MixerList {

    pub fn new(effects: Vec<Effect>) -> MixerList {

        let mut list = MixerList {
            effects: effects,
            source: None
        };

        list.effects.reverse();
        list.update();
        list

    }

    pub fn get_active_source(&mut self) -> Option<&mut MixerSource> {
        self.source.as_mut()
    }

    pub fn update(&mut self) {

        // Check if there is either no current source or the current source has ended
        if self.source.is_none() || !self.source.as_ref().unwrap().is_active() {

            // Check if there is another, valid file which should be played
            while let Some(effect) = self.effects.pop() {

                // Use the first valid file as the next source
                if let Ok(source) = MixerSource::new(effect) {
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

