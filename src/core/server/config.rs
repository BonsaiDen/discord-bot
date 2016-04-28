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


// Server Config Abstraction --------------------------------------------------
pub struct Config {
    server_id: ServerId,
    config_directory: PathBuf,
    config_file: PathBuf
}

impl Config {

    pub fn new(server_id: ServerId, config_directory: PathBuf) -> Config {

        let mut config_file = config_directory.clone();
        config_file.push(server_id.0.to_string());
        config_file.set_extension("toml");

        Config {
            server_id: server_id,
            config_directory: config_directory,
            config_file: config_file
        }

    }

    pub fn load(&mut self) -> Option<(
        HashMap<String, Vec<String>>,
        HashMap<String, Greeting>
    )> {
        if self.ensure_directory() {
            match File::open(self.config_file.clone()) {
                Ok(mut file) => {

                    let mut toml = String::new();
                    file.read_to_string(&mut toml).ok();

                    if let Some(value) = toml::Parser::new(&toml).parse() {
                        info!("[Config] [{}] [Load] Configuration loaded successfully.", self);
                        Some(parse_toml(value))

                    } else {
                        info!("[Config] [{}] [Load] Configuration file could not be parsed.", self);
                        None
                    }

                }
                Err(err) => {
                    info!("[Config] [{}] [Load] Failed to load configuration file: {}", self, err);
                    None
                }
            }

        } else {
            None
        }
    }

    pub fn store(
        &self,
        aliases: &HashMap<String, Vec<Effect>>,
        greetings: &HashMap<String, Greeting>
    ) {
        if self.ensure_directory() {
            match File::create(self.config_file.clone()) {
                Ok(mut f) => {

                    let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

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

                    if let Err(err) = write!(f, "{}", toml::Value::Table(toml)) {
                        info!("[Config] [{}] [Store] Failed to write configuration file: {}", self, err);

                    } else {
                        info!("[Config] [{}] [Store] Configuration stored successfully.", self);
                    }

                }
                Err(err) => {
                    info!("[Config] [{}] [Store] Failed to create configuration file: {}", self, err);
                }
            }
        }
    }

    fn ensure_directory(&self) -> bool {
        if let Err(err) = fs::create_dir_all(self.config_directory.clone()) {
            info!("[Config] [{}] Failed to create configuration directory: {}", self, err);
            false

        } else {
            true
        }
    }

}


// Helpers --------------------------------------------------------------------
fn parse_toml(value: BTreeMap<String, toml::Value>) -> (
    HashMap<String, Vec<String>>,
    HashMap<String, Greeting>
) {

    let mut aliases = HashMap::new();
    if let Some(&toml::Value::Table(ref table)) = value.get("aliases") {
        for (alias, names) in table {
            if let &toml::Value::Array(ref names) = names {
                let mut effects: Vec<String> = Vec::new();
                for name in names {
                    if let &toml::Value::String(ref name) = name {
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
            if let &toml::Value::String(ref effect) = effect {
                greetings.insert(
                    nickname.clone(),
                    Greeting::new(nickname.clone(), effect.clone(), true)
                );
            }
        }
    }

    (aliases, greetings)

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.server_id.0)
    }
}

