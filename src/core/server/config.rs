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
use super::super::Effect;
use super::super::voice::Greeting;


// Type Aliases ---------------------------------------------------------------
pub type ConfigData = (HashMap<String, Vec<String>>, HashMap<String, Greeting>, Vec<String>);


// Server Config Abstraction --------------------------------------------------
pub struct Config {
    server_id: ServerId,
    config_directory: PathBuf,
    config_file: PathBuf
}

impl Config {

    pub fn new(server_id: ServerId, mut config_directory: PathBuf) -> Config {

        config_directory.push(server_id.0.to_string());

        let mut config_file = config_directory.clone();
        config_file.push("config");
        config_file.set_extension("toml");

        Config {
            server_id: server_id,
            config_directory: config_directory,
            config_file: config_file
        }

    }

    pub fn load(&mut self) -> Result<ConfigData, String> {
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

    pub fn store(
        &self,
        aliases: &HashMap<String, Vec<Effect>>,
        greetings: &HashMap<String, Greeting>,
        admins: &[String]

    ) -> Result<(), String> {
        self.create_config_dir()
            .and_then(|_| {
                File::create(self.config_file.clone())
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        write!(
                            file,
                            "{}",
                            encode_toml(aliases, greetings, admins)

                        ).map_err(|err| err.to_string())
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
fn decode_toml(value: BTreeMap<String, toml::Value>) -> ConfigData {

    let mut aliases = HashMap::new();
    if let Some(&toml::Value::Table(ref table)) = value.get("aliases") {
        for (alias, names) in table {
            if let toml::Value::Array(ref names) = *names {
                let mut effects: Vec<String> = Vec::new();
                for name in names {
                    if let toml::Value::String(ref name) = *name {
                        effects.push(name.clone());
                    }
                }
                aliases.insert(alias.clone(), effects);
            }
        }
    }

    let mut greetings = HashMap::new();
    if let Some(&toml::Value::Table(ref table)) = value.get("greetings") {
        for (nickname, effect) in table {
            if let toml::Value::String(ref effect) = *effect {
                greetings.insert(
                    nickname.clone(),
                    Greeting::new(nickname.clone(), effect.clone(), true)
                );
            }
        }
    }

    let mut admins = Vec::new();
    if let Some(&toml::Value::Array(ref nicknames)) = value.get("admins") {
        for nickname in nicknames {
            if let toml::Value::String(ref nickname) = *nickname {
                admins.push(nickname.clone());
            }
        }
    }

    (aliases, greetings, admins)

}

// toml::Value::Table(toml)
fn encode_toml(
    aliases: &HashMap<String, Vec<Effect>>,
    greetings: &HashMap<String, Greeting>,
    admins: &[String]

) -> toml::Value {

    let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

    let list = toml::Value::Array(admins.iter().map(|nickname| {
        toml::Value::String(nickname.to_string())

    }).collect());

    toml.insert("admins".to_string(), list);

    let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
    for (nickname, greeting) in greetings {
        if greeting.permanent {
            table.insert(
                nickname.clone(),
                toml::Value::String(greeting.effect.clone())
            );
        }
    }
    toml.insert("greetings".to_string(), toml::Value::Table(table));

    let mut table: BTreeMap<String, toml::Value> = BTreeMap::new();
    for (alias, effects) in aliases {
        table.insert(
            alias.clone(),
            toml::Value::Array(effects.iter().map(|e| {
                toml::Value::String(e.name().to_string())

            }).collect())
        );
    }
    toml.insert("aliases".to_string(), toml::Value::Table(table));

    toml::Value::Table(toml)

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.server_id.0)
    }
}

