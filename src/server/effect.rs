// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;


// Internal Dependencies ------------------------------------------------------
use ::audio::MixerCommand;
use ::action::ActionHandler;
use ::bot::BotConfig;
use ::core::EventQueue;
use ::effect::Effect;
use super::Server;


// Server Effect Interface ----------------------------------------------------
impl Server {

    pub fn play_effects(
        &mut self,
        channel_id: &ChannelId,
        effects: &[Effect],
        queued: bool,
        queue: &mut EventQueue,
        complete_action: Option<Box<ActionHandler>>
    ) {

        let (has_channel, bitrate) = if let Some(channel) = self.channels.get(channel_id) {

            // When pinned, only play effects for the pinned channel
            if let Some(pinned_channel_id) = self.pinned_channel_id {
                if *channel_id == pinned_channel_id {
                    info!("{} {} playing {} effect(s)...", self, channel, effects.len());
                    (true, channel.bitrate())

                } else {
                    info!("{} {} not playing effect(s), pinned to another channel.", self, channel);
                    (false, 0)
                }

            } else {
                info!("{} {} playing {} effect(s)...", self, channel, effects.len());
                (true, channel.bitrate())
            }

        } else {
            (false, 0)
        };

        // See if we're actually in the requested channel or if we're going
        // to switch there.
        //
        // We want to prevent playing greetings from other channels in case
        // we are pinned to another channel already.
        if has_channel && self.join_voice(channel_id, queue) {

            // Add playback IDs to list of effects
            let mut effects: Vec<(Effect, Option<Box<ActionHandler>>)> = effects.into_iter().map(|effect| {

                // Mark effect as played
                self.effects.played_effect(&effect.name);

                (effect.clone_with_bitrate(bitrate), None)

            }).collect();

            // Attach completion action to last effect
            if let Some(last_effect) = effects.last_mut() {
                last_effect.1 = complete_action;
            }

            if let Some(queue) = self.mixer_commands.as_mut() {
                queue.send(if queued {
                    MixerCommand::QueueEffects(effects)

                } else {
                    MixerCommand::PlayEffects(effects)

                }).ok();
            }

        }

    }

    pub fn silence_active_effects(&mut self) {
        if let Some(queue) = self.mixer_commands.as_mut() {
            queue.send(MixerCommand::ClearQueue).ok();
        }
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.effects.has_effect(effect_name)
    }

    pub fn get_effect(&self, effect_name: &str) -> Option<&Effect> {
        self.effects.get_effect(effect_name)
    }

    pub fn has_matching_effects(&self, effect_name: &str, bot_config: &BotConfig) -> bool {
        !self.map_effects(
            &[effect_name.to_string()],
            true,
            bot_config

        ).is_empty()
    }

    pub fn map_effects(
        &self,
        patterns: &[String],
        match_all: bool,
        bot_config: &BotConfig

    ) -> Vec<&Effect> {
        self.effects.map_patterns(
            patterns,
            Some(&self.config.aliases),
            match_all,
            bot_config
        )
    }

    pub fn map_similiar_effects(&self, patterns: &[String]) -> Vec<&str> {
        self.effects.map_similiar(
            patterns,
            &self.config.aliases
        )
    }

    pub fn rename_effect(&mut self, effect: &Effect, effect_name: &str) -> Result<(), String> {
        self.effects.rename_effect(&self.config, effect, effect_name)
    }

    pub fn delete_effect(&mut self, effect: &Effect) -> Result<(), String> {
        self.effects.delete_effect(&self.config, effect)
    }

    pub fn download_effect(
        &mut self,
        effect_name: &str,
        upload_url: &str,
        uploader: &str

    ) -> Result<(), String> {
        self.effects.download_effect(
            &self.config,
            effect_name,
            upload_url,
            uploader
        )
    }

    pub fn download_transcript(
        &mut self,
        effect_name: &str,
        upload_url: &str

    ) -> Result<(), String> {
        self.effects.download_transcript(
            &self.config,
            effect_name,
            upload_url
        )
    }

}

