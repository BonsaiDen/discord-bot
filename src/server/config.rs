// STD Dependencies -----------------------------------------------------------
use std::env;
use std::fmt;
use std::path::PathBuf;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// External Dependencies ------------------------------------------------------
use diesel::Connection;
use diesel::sqlite::SqliteConnection;
use diesel::connection::SimpleConnection;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Server Configuration Abstraction -------------------------------------------
pub struct ServerConfig {
    pub table_id: String,
    pub connection: SqliteConnection,
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
            table_id: format!("{}", server_id),
            connection: establish_connection().expect("Failed to establish database connection."),
            effects_path: effects_path,
            recordings_path: recordings_path
        }

    }

}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ServerConfig]")
    }
}


// Helpers --------------------------------------------------------------------
fn establish_connection() -> Result<SqliteConnection, String> {
    env::var("DATABASE_URL").map_err(|err| {
        err.to_string()

    }).and_then(|url| {
        SqliteConnection::establish(&url).map_err(|err| {
            err.to_string()
        })

    }).and_then(|connection| {
        connection.batch_execute(
            "PRAGMA synchronous = OFF; PRAGMA journal_mode = MEMORY;"

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            Ok(connection)
        })
    })
}

