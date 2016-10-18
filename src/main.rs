#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(slice_patterns)]

// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate log;
extern crate flac;
extern crate toml;
extern crate rand;
extern crate hyper;
extern crate chrono;
extern crate dotenv;
extern crate discord;
extern crate clock_ticks;


// STD Dependencies -----------------------------------------------------------
use std::env;
use std::path::PathBuf;


// External Dependencies ------------------------------------------------------
use dotenv::dotenv;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
mod action;
mod audio;
mod bot;
mod command;
mod core;
mod effect;
mod logger;
mod upload;
mod text_util;


// Main -----------------------------------------------------------------------
fn main() {

    // Load environment config
    dotenv().ok();

    logger::Logger::init().ok();

    let token = env::var("DISCORD_BOT_TOKEN").unwrap_or("".into());
    let config = bot::BotConfig {
        bot_nickname: env::var("DISCORD_BOT_NICKNAME").unwrap_or("".into()),
        server_whitelist: env::var("SERVER_WHITELIST").ok().and_then(|servers| {
            Some(servers.split(',').map(|id| {
                ServerId(id.parse().unwrap_or(0))

            }).collect::<Vec<ServerId>>())

        }).unwrap_or_else(Vec::new),
        config_path: PathBuf::from(
            env::var_os("CONFIG_DIRECTORY").unwrap_or("".into())
        ),
        effect_playback_separation_ms: env::var("EFFECT_PLAYBACK_SEPARATION").unwrap_or("".into()).parse().unwrap_or(10000),
        greeting_separation_ms: env::var("USER_GREETING_SERPARATION").unwrap_or("".into()).parse().unwrap_or(30000),
        flac_max_file_size: env::var("FLAC_MAX_FILE_SIZE").unwrap_or("".into()).parse().unwrap_or(2048 * 1024),
        flac_sample_rate: 48000,
        flac_bits_per_sample: 16
    };

    bot::Bot::create(token, config);

}

