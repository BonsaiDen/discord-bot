// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// Stream Online Check Implementation -----------------------------------------
pub struct Action {
    server_id: ServerId
}

impl Action {
    pub fn new(server_id: ServerId) -> Box<Action> {
        Box::new(Action {
            server_id: server_id
        })
    }
}

impl ActionHandler for Action {

    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.server_id) {
            println!("Check Stream online status for {:?}", self.server_id);
            for streamer in server.list_streamers() {
                println!("Checking {}...", streamer.twitch_nick);
            }
        }

        vec![]

    }

}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[OnlineCheckAction]")
    }
}

