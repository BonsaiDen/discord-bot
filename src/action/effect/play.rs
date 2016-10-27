// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::core::EventQueue;
use ::bot::{Bot, BotConfig};
use ::action::{ActionHandler, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct Action {
    server_id: ServerId,
    channel_id: ChannelId,
    effects: Vec<Effect>,
    queued: bool
}

impl Action {
    pub fn new(
        server_id: ServerId,
        channel_id: ChannelId,
        effects: Vec<&Effect>,
        queued: bool

    ) -> Box<Action> {
        Box::new(Action {
            server_id: server_id,
            channel_id: channel_id,
            effects: effects.iter().map(|e| (*e).clone()).collect(),
            queued: queued
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
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

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [PlayEffects]")
    }
}

