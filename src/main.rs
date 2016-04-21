// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate log;
extern crate chrono;
extern crate discord;
extern crate rand;
extern crate dotenv;


// STD Dependencies -----------------------------------------------------------
use std::env;


// External Dependencies ------------------------------------------------------
use dotenv::dotenv;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
mod core;
mod logger;


// Main -----------------------------------------------------------------------
fn main() {

    // Load environment config
    dotenv().ok();

    logger::Logger::init().ok();

    let mut bot = core::Bot::new(
        env::var("DISCORD_BOT_TOKEN").unwrap_or("".into()),
        env::var("SERVER_WHITELIST").ok().and_then(|servers| {
            Some(servers.split(",").map(|id| {
                ServerId(id.parse().unwrap_or(0))

            }).collect::<Vec<ServerId>>())
        })
    );

    bot.connect();

}

