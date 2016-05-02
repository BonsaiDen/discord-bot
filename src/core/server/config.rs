// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::fmt;
use std::path::PathBuf;
use std::collections::{BTreeMap, HashMap};


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// External Dependencies ------------------------------------------------------
use toml;


// Internal Dependencies ------------------------------------------------------
use super::super::voice::Greeting;


// Config Structure -----------------------------------------------------------
pub struct Config {
    pub aliases: HashMap<String, Vec<String>>,
    pub greetings: HashMap<String, Greeting>,
    pub uploaders: Vec<String>,
    pub admins: Vec<String>
}

pub struct ConfigRef<'a> {
    pub aliases: &'a HashMap<String, Vec<String>>,
    pub greetings: &'a HashMap<String, Greeting>,
    pub uploaders: &'a [String],
    pub admins: &'a [String]
}


// Server ConfigHandler Abstraction -------------------------------------------
pub struct ConfigHandler {
    server_id: ServerId,
    config_directory: PathBuf,
    config_file: PathBuf
}

impl ConfigHandler {

    pub fn new(server_id: ServerId, mut config_directory: PathBuf) -> ConfigHandler {

        config_directory.push(server_id.0.to_string());

        let mut config_file = config_directory.clone();
        config_file.push("config");
        config_file.set_extension("toml");

        ConfigHandler {
            server_id: server_id,
            config_directory: config_directory,
            config_file: config_file
        }

    }

    pub fn load(&mut self) -> Result<Config, String> {
        self.create_config_dir()
            .and_then(|_| {
                File::open(self.config_file.clone())
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)
                            .map_err(|err| err.to_string())
                            .map(|_| buffer)

                    })
            })
            .and_then(|buffer| {
                toml::Parser::new(&buffer)
                    .parse()
                    .map_or_else(|| {
                        Err("Failed to parse configuration toml.".to_string())

                    }, |value|{
                        Ok(decode_toml(value))
                    })
            })
    }

    pub fn store<'a>(&self, config: ConfigRef<'a>) -> Result<(), String> {
        self.create_config_dir()
            .and_then(|_| {
                File::create(self.config_file.clone())
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        write!(file, "{}", encode_toml(config))
                            .map_err(|err| err.to_string())
                    })
            })
    }

    fn create_config_dir(&self) -> Result<(), String> {
        fs::create_dir_all(
            self.config_directory.clone()

        ).map_err(|err| err.to_string())
    }

}


// Helpers --------------------------------------------------------------------
fn decode_toml(value: BTreeMap<String, toml::Value>) -> Config {

    let mut config = Config {
        aliases: HashMap::new(),
        greetings: HashMap::new(),
        admins: Vec::new(),
        uploaders: Vec::new()
    };

    if let Some(&toml::Value::Table(ref table)) = value.get("aliases") {
        for (alias, names) in table {
            if let toml::Value::Array(ref names) = *names {
                let mut effects: Vec<String> = Vec::new();
                for name in names {
                    if let toml::Value::String(ref name) = *name {
                        effects.push(name.clone());
                    }
                }
                config.aliases.insert(alias.clone(), effects);
            }
        }
    }

    if let Some(&toml::Value::Table(ref table)) = value.get("greetings") {
        for (nickname, effect) in table {
            if let toml::Value::String(ref effect) = *effect {
                config.greetings.insert(
                    nickname.clone(),
                    Greeting::new(nickname.clone(), effect.clone(), true)
                );
            }
        }
    }

    if let Some(&toml::Value::Array(ref nicknames)) = value.get("admins") {
        for nickname in nicknames {
            if let toml::Value::String(ref nickname) = *nickname {
                config.admins.push(nickname.clone());
            }
        }
    }

    if let Some(&toml::Value::Array(ref nicknames)) = value.get("uploaders") {
        for nickname in nicknames {
            if let toml::Value::String(ref nickname) = *nickname {
                config.uploaders.push(nickname.clone());
            }
        }
    }

    config

}

fn encode_toml<'a>(config: ConfigRef<'a>) -> toml::Value {

    // TODO clean up
    let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

    let list = toml::Value::Array(config.admins.iter().map(|nickname| {
        toml::Value::String(nickname.to_string())

    }).collect());

    toml.insert("admins".to_string(), list);

    let list = toml::Value::Array(config.uploaders.iter().map(|nickname| {
        toml::Value::String(nickname.to_string())

    }).collect());

    toml.insert("uploaders".to_string(), list);

    let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
    for (nickname, greeting) in config.greetings {
        if greeting.permanent {
            table.insert(
                nickname.clone(),
                toml::Value::String(greeting.effect.clone())
            );
        }
    }
    toml.insert("greetings".to_string(), toml::Value::Table(table));

    let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
    for (alias, effects) in config.aliases {
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


// Traits  --------------------------------------------------------------------
impl fmt::Display for ConfigHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.server_id.0)
    }
}

