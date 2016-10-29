// Internal Dependencies ------------------------------------------------------
use super::MixerSource;
use ::effect::Effect;


// Mixer Source List Implementation -------------------------------------------
pub struct MixerList {
    effects: Vec<(Effect, usize)>,
    source: Option<MixerSource>
}

impl MixerList {

    pub fn new(effects: Vec<(Effect, usize)>) -> MixerList {

        let mut list = MixerList {
            effects: effects,
            source: None
        };

        list.effects.reverse();
        list.udpate_and_complete();
        list

    }

    pub fn get_active_source(&mut self) -> Option<&mut MixerSource> {
        self.source.as_mut()
    }

    pub fn udpate_and_complete(&mut self) -> Option<(Effect, usize)> {

        let mut completed_effect = None;

        // Check whether there is either no current source or the current source
        // has completed playback
        if !self.is_active() {

            // Extract effect from the last source that completed playback
            if let Some(source) = self.source.take() {
                completed_effect = Some(source.into_effect());
            }

            // Check if there is another, valid effect which should be played
            while let Some(effect) = self.effects.pop() {

                // Use the first valid file as the next source
                if let Ok(source) = MixerSource::new(effect.0, effect.1) {
                    self.source = Some(source);
                    break;
                }

            }
        }

        completed_effect

    }

    pub fn is_active(&self) -> bool {
        if let Some(ref source) = self.source {
            source.is_active()

        } else {
            false
        }
    }

}

