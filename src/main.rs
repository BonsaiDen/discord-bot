#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate log;
extern crate chrono;
extern crate discord;
extern crate rand;
extern crate dotenv;
extern crate flac;
extern crate hyper;
extern crate toml;
extern crate edit_distance;


// STD Dependencies -----------------------------------------------------------
use std::env;
use std::path::PathBuf;


// External Dependencies ------------------------------------------------------
use dotenv::dotenv;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
mod core;
mod logger;
mod util;


// Main -----------------------------------------------------------------------
fn main() {

    // Load environment config
    dotenv().ok();

    logger::Logger::init().ok();

    let mut bot = core::Bot::new(
        env::var("DISCORD_BOT_TOKEN").unwrap_or("".into()),
        env::var("SERVER_WHITELIST").ok().and_then(|servers| {
            Some(servers.split(',').map(|id| {
                ServerId(id.parse().unwrap_or(0))

            }).collect::<Vec<ServerId>>())
        }),
        PathBuf::from(env::var_os("EFFECTS_DIRECTORY").unwrap_or("".into())),
        PathBuf::from(env::var_os("CONFIG_DIRECTORY").unwrap_or("".into()))
    );

    bot.connect();

}

