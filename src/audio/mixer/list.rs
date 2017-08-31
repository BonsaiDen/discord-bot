// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::action::ActionOption;
use ::effect::Effect;
use super::MixerSource;


// Mixer Source List Implementation -------------------------------------------
pub struct MixerList {
    effects: Vec<(Effect, ActionOption)>,
    source: Option<MixerSource>
}

impl MixerList {

    pub fn new(effects: Vec<(Effect, ActionOption)>) -> MixerList {

        let mut list = MixerList {
            effects: effects,
            source: None
        };

        list.effects.reverse();
        list.update_and_complete();
        list

    }

    pub fn clear(&mut self) -> Vec<(Effect, ActionOption)> {
        self.effects.drain(0..).collect()
    }

    pub fn get_active_source(&mut self) -> Option<&mut MixerSource> {
        self.source.as_mut()
    }

    pub fn update_and_complete(&mut self) -> Option<(Effect, ActionOption)> {

        let mut completed_effect = None;

        // Check whether there is either no current source or the current source
        // has completed playback
        if !self.is_active() {

            // Extract effect from the last source that completed playback
            if let Some(source) = self.source.take() {
                completed_effect = Some(source.into_effect());
            }

            // Check if there is another, valid effect which should be played
            if let Some(effect) = self.effects.pop() {

                match MixerSource::new(effect.0, effect.1) {

                    // Use the first valid effect as the next source
                    Ok(source) => {
                        self.source = Some(source);
                    },

                    // If we failed to load the effect we return it immediately
                    // so the list doesn't get stuck
                    Err(effect) => {
                        warn!("{} Failed to load effect {:?}", self, effect.0);
                        completed_effect = Some(effect);
                    }
                };

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

impl fmt::Display for MixerList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[MixerList {} effect(s)]", self.effects.len())
    }
}

