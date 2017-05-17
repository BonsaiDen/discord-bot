#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Crates ---------------------------------------------------------------------
#[macro_use] extern crate log;
extern crate flac;
extern crate rand;
#[macro_use] extern crate hyper;
extern crate chrono;
extern crate dotenv;
extern crate rayon;
extern crate discord;
extern crate app_dirs;
extern crate vorbis_enc;
extern crate clock_ticks;
extern crate edit_distance;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;


// STD Dependencies -----------------------------------------------------------
use std::env;


// External Dependencies ------------------------------------------------------
use dotenv::dotenv;
use app_dirs::{AppInfo, AppDataType};


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Internal Dependencies ------------------------------------------------------
mod action;
mod audio;
#[macro_use]
mod command;
mod core;
mod effect;
mod server;
mod upload;

mod db;
mod bot;
mod logger;
mod text_util;


// Statics --------------------------------------------------------------------
const APP_INFO: AppInfo = AppInfo {
    name: "discord-bot",
    author: "BonsaiDen"
};


// Main -----------------------------------------------------------------------
fn main() {

    // Load environment config
    dotenv().ok();

    logger::Logger::init().ok();

    let token = env::var("DISCORD_BOT_TOKEN").unwrap_or_else(|_| "".into());
    let config = bot::BotConfig {
        bot_nickname: env::var("DISCORD_BOT_NICKNAME").unwrap_or_else(|_| "".into()),
        server_whitelist: env::var("SERVER_WHITELIST").ok().and_then(|servers| {
            Some(servers.split(',').map(|id| {
                ServerId(id.parse().unwrap_or(0))

            }).collect::<Vec<ServerId>>())

        }).unwrap_or_else(Vec::new),
        config_path: app_dirs::app_root(AppDataType::UserConfig, &APP_INFO).expect("Failed to retrieve configuration directory."),
        twitch_client_id: env::var("TWITCH_CLIENT_ID").unwrap_or_else(|_| "".into()),
        twitch_update_interval: env::var("TWITCH_UPDATE_INTERVAL").unwrap_or_else(|_| "".into()).parse().unwrap_or(0),
        effect_playback_separation_ms: env::var("EFFECT_PLAYBACK_SEPARATION").unwrap_or_else(|_| "".into()).parse().unwrap_or(10000),
        greeting_separation_ms: env::var("USER_GREETING_SERPARATION").unwrap_or_else(|_| "".into()).parse().unwrap_or(30000),
        flac_max_file_size: env::var("FLAC_MAX_FILE_SIZE").unwrap_or_else(|_| "".into()).parse().unwrap_or(2048 * 1024),
        flac_sample_rate: 48000,
        flac_bits_per_sample: 16
    };

    bot::Bot::create(token, config);

}

