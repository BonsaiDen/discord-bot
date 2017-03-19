// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;



// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Server Configuration Abstraction -------------------------------------------
#[derive(Debug)]
pub struct ServerConfig {
    pub effects_path: PathBuf,
    pub recordings_path: PathBuf
}

impl ServerConfig {

    pub fn new(server_id: &ServerId, bot_config: &BotConfig) -> Self {

        let mut effects_path = bot_config.config_path.clone();
        effects_path.push(server_id.0.to_string());
        effects_path.push("effects");

        let mut recordings_path = bot_config.config_path.clone();
        recordings_path.push(server_id.0.to_string());
        recordings_path.push("recordings");

        ServerConfig {
            effects_path: effects_path,
            recordings_path: recordings_path
        }

    }

}

