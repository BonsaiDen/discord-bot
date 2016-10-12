// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;
use ::bot::{Bot, BotConfig};
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct PlayEffects {
    server_id: ServerId,
    channel_id: ChannelId,
    effects: Vec<Effect>,
    queued: bool
}

impl PlayEffects {
    pub fn new(
        server_id: ServerId,
        channel_id: ChannelId,
        effects: Vec<Effect>,
        queued: bool

    ) -> Box<PlayEffects> {
        Box::new(PlayEffects {
            server_id: server_id,
            channel_id: channel_id,
            effects: effects,
            queued: queued
        })
    }
}

impl Action for PlayEffects {
    fn run(&self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        if let Some(server) = bot.get_server(&self.server_id) {
            server.play_effects(
                &self.channel_id,
                &self.effects,
                self.queued,
                queue
            );
            vec![]

        } else {
            vec![]
        }
    }
}

impl fmt::Display for PlayEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [PlayEffects]")
    }
}

