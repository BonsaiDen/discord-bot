// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::path::PathBuf;
use std::collections::{HashMap, BTreeMap};


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// External Dependencies ------------------------------------------------------
use toml;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Server Configuration Abstraction -------------------------------------------
#[derive(Debug)]
pub struct ServerConfig {
    pub config_path: PathBuf,
    pub effects_path: PathBuf,
    pub aliases: HashMap<String, Vec<String>>,
    pub greetings: HashMap<String, String>,
    pub uploaders: Vec<String>,
    pub admins: Vec<String>,
    pub banned: Vec<String>
}

impl ServerConfig {

    pub fn new(server_id: &ServerId, bot_config: &BotConfig) -> Self {

        let mut config_path = bot_config.config_path.clone();
        config_path.push(server_id.0.to_string());
        config_path.push("config");
        config_path.set_extension("toml");

        let mut effects_path = bot_config.config_path.clone();
        effects_path.push(server_id.0.to_string());
        effects_path.push("effects");

        ServerConfig {
            config_path: config_path,
            effects_path: effects_path,
            aliases: HashMap::new(),
            greetings: HashMap::new(),
            admins: Vec::new(),
            uploaders: Vec::new(),
            banned: Vec::new()
        }

    }

    pub fn ensure_directory(&self) -> Result<(), String> {
        fs::create_dir_all(
            self.config_path.clone().parent().unwrap()

        ).map_err(|err| err.to_string())
    }

    pub fn encode_to_toml(&self) -> toml::Value {

        let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

        // TODO dry
        let list = toml::Value::Array(self.admins.iter().map(|nickname| {
            toml::Value::String(nickname.to_string())

        }).collect());

        toml.insert("admins".to_string(), list);

        let list = toml::Value::Array(self.uploaders.iter().map(|nickname| {
            toml::Value::String(nickname.to_string())

        }).collect());

        toml.insert("uploaders".to_string(), list);

        let list = toml::Value::Array(self.banned.iter().map(|nickname| {
            toml::Value::String(nickname.to_string())

        }).collect());

        toml.insert("banned".to_string(), list);

        let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
        for (nickname, effect) in &self.greetings {
            table.insert(
                nickname.clone(),
                toml::Value::String(effect.clone())
            );
        }
        toml.insert("greetings".to_string(), toml::Value::Table(table));

        let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
        for (alias, effects) in &self.aliases {
            table.insert(
                alias.clone(),
                toml::Value::Array(effects.iter().map(|e| {
                    toml::Value::String(e.to_string())

                }).collect())
            );
        }
        toml.insert("aliases".to_string(), toml::Value::Table(table));

        toml::Value::Table(toml)

    }

    pub fn decode_from_toml(&mut self, value: BTreeMap<String, toml::Value>) {

        self.aliases.clear();
        self.greetings.clear();
        self.admins.clear();
        self.uploaders.clear();

        if let Some(&toml::Value::Table(ref table)) = value.get("aliases") {
            for (alias, names) in table {
                if let toml::Value::Array(ref names) = *names {
                    let mut effects: Vec<String> = Vec::new();
                    for name in names {
                        if let toml::Value::String(ref name) = *name {
                            effects.push(name.clone());
                        }
                    }
                    self.aliases.insert(alias.clone(), effects);
                }
            }
        }

        if let Some(&toml::Value::Table(ref table)) = value.get("greetings") {
            for (nickname, effect) in table {
                if let toml::Value::String(ref effect) = *effect {
                    self.greetings.insert(
                        nickname.clone(),
                        effect.clone()
                    );
                }
            }
        }

        // TODO dry
        if let Some(&toml::Value::Array(ref nicknames)) = value.get("admins") {
            for nickname in nicknames {
                if let toml::Value::String(ref nickname) = *nickname {
                    self.admins.push(nickname.clone());
                }
            }
        }

        if let Some(&toml::Value::Array(ref nicknames)) = value.get("uploaders") {
            for nickname in nicknames {
                if let toml::Value::String(ref nickname) = *nickname {
                    self.uploaders.push(nickname.clone());
                }
            }
        }

        if let Some(&toml::Value::Array(ref nicknames)) = value.get("banned") {
            for nickname in nicknames {
                if let toml::Value::String(ref nickname) = *nickname {
                    self.banned.push(nickname.clone());
                }
            }
        }

    }

}

