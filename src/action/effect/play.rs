// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::core::EventQueue;
use ::bot::{Bot, BotConfig};
use ::action::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    server_id: ServerId,
    channel_id: ChannelId,
    effects: Vec<Effect>,
    queued: bool
}

impl ActionImpl {
    pub fn new(
        server_id: ServerId,
        channel_id: ChannelId,
        effects: Vec<&Effect>,
        queued: bool

    ) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            server_id: server_id,
            channel_id: channel_id,
            effects: effects.iter().map(|e| (*e).clone()).collect(),
            queued: queued
        })
    }
}

impl Action for ActionImpl {
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

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [PlayEffects]")
    }
}

