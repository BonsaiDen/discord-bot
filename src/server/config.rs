// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::collections::{HashMap, BTreeMap};


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// External Dependencies ------------------------------------------------------
use toml;
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Server Configuration Abstraction -------------------------------------------
#[derive(Debug)]
pub struct ServerConfig {
    config_path: PathBuf,
    server_id: ServerId,
    pub effects_path: PathBuf,
    pub recordings_path: PathBuf,
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

        let mut recordings_path = bot_config.config_path.clone();
        recordings_path.push(server_id.0.to_string());
        recordings_path.push("recordings");

        ServerConfig {
            server_id: *server_id,
            config_path: config_path,
            effects_path: effects_path,
            recordings_path: recordings_path,
            aliases: HashMap::new(),
            greetings: HashMap::new(),
            admins: Vec::new(),
            uploaders: Vec::new(),
            banned: Vec::new()
        }

    }

    pub fn sync_to_db(&self, connection: &SqliteConnection) {

        let sid = format!("{}", self.server_id);
        info!("Synchronizing configuration for server {} to sqlite database...", sid);

        {
            use ::db::models::NewAlias;
            use ::db::schema::aliases::dsl::{aliases, server_id};

            // Delete existing aliases
            diesel::delete(aliases.filter(server_id.eq(&sid)))
                   .execute(connection)
                   .expect("Error deleting aliases.");

            // Insert new aliases
            for (alias_name, mapped_names) in &self.aliases {

                use ::db::schema::aliases;

                let names = mapped_names.join(" ");
                let alias = NewAlias {
                    server_id: &sid,
                    name: &alias_name,
                    effect_names: &names
                };

                diesel::insert(&alias).into(aliases::table)
                       .execute(connection)
                       .expect("Error saving new alias.");

            }
        }

        {
            use ::db::models::NewGreeting;
            use ::db::schema::greetings::dsl::{greetings, server_id};

            // Delete existing users
            diesel::delete(greetings.filter(server_id.eq(&sid)))
                   .execute(connection)
                   .expect("Error deleting greeting.");

            // Insert new greetings
            for (user_name, mapped_name) in &self.greetings {

                use ::db::schema::greetings;

                let alias = NewGreeting {
                    server_id: &sid,
                    nickname: &user_name,
                    effect_name: &mapped_name
                };

                diesel::insert(&alias).into(greetings::table)
                       .execute(connection)
                       .expect("Error saving new greeting.");

            }
        }

        {
            use ::db::models::NewUser;
            use ::db::schema::users::dsl::{users, server_id};

            // Delete existing users
            diesel::delete(users.filter(server_id.eq(&sid)))
                   .execute(connection)
                   .expect("Error deleting users.");

            let mut new_users: HashMap<String, NewUser> = HashMap::new();

            for nickname in &self.admins {
                new_users.entry(nickname.clone()).or_insert_with(|| {
                    NewUser {
                        server_id: &sid,
                        nickname: nickname,
                        is_admin: false,
                        is_uploader: false,
                        is_banned: false
                    }

                }).is_admin = true;
            }

            for nickname in &self.uploaders {
                new_users.entry(nickname.clone()).or_insert_with(|| {
                    NewUser {
                        server_id: &sid,
                        nickname: nickname,
                        is_admin: false,
                        is_uploader: false,
                        is_banned: false
                    }

                }).is_uploader = true;
            }

            for nickname in &self.banned {
                new_users.entry(nickname.clone()).or_insert_with(|| {
                    NewUser {
                        server_id: &sid,
                        nickname: nickname,
                        is_admin: false,
                        is_uploader: false,
                        is_banned: false
                    }

                }).is_banned = true;
            }

            for new_user in new_users.values() {
                use ::db::schema::users;
                diesel::insert(new_user).into(users::table)
                       .execute(connection)
                       .expect("Error saving new user.");
            }

        }

    }

    pub fn load(&mut self, server_name: &str) -> Result<(), String> {
        self.ensure_defaults(server_name)
            .and_then(|_| {
                info!(
                    "{} Reading configuration toml: {}",
                    server_name,
                    self.config_path.to_str().unwrap()
                );
                File::open(self.config_path.clone())
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

                    }, |value| {
                        self.decode_from_toml(value);
                        Ok(())
                    })
            })
    }

    pub fn store(&mut self, server_name: &str) -> Result<(), String> {
        self.ensure_defaults(server_name)
            .and_then(|config_path| {
                info!(
                    "{} Writing configuration toml: {}",
                    server_name,
                    config_path.to_str().unwrap()
                );
                File::create(config_path)
                    .map_err(|err| err.to_string())
                    .and_then(|mut file| {
                        write!(file, "{}", self.encode_to_toml())
                            .map_err(|err| err.to_string())
                    })
            })
    }

    fn ensure_defaults(&self, server_name: &str) -> Result<PathBuf, String> {
        fs::create_dir_all(
            self.config_path.clone().parent().expect("Could not create server configuration directory.")

        ).map_err(|err| {
            err.to_string()

        }).and_then(|_| {
            if let Ok(_) = File::open(self.config_path.clone()) {
                Ok(self.config_path.clone())

            } else {
                match File::create(self.config_path.clone()) {
                    Ok(_) => {
                        info!(
                            "[{}] Created initial configuration toml: {}",
                            server_name,
                            self.config_path.to_str().unwrap()
                        );
                        Ok(self.config_path.clone())
                    },
                    Err(err) => Err(
                        format!(
                            "Failed to create initial configuration toml ({}): {}",
                            self.config_path.to_str().unwrap(),
                            err.to_string()
                        )
                    )
                }
            }
        })
    }

    fn encode_to_toml(&self) -> toml::Value {

        let mut toml: BTreeMap<String, toml::Value> = BTreeMap::new();

        toml.insert("admins".to_string(), to_toml_strings(&self.admins));
        toml.insert("uploaders".to_string(), to_toml_strings(&self.uploaders));
        toml.insert("banned".to_string(), to_toml_strings(&self.banned));

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
            table.insert(alias.clone(), to_toml_strings(effects));
        }
        toml.insert("aliases".to_string(), toml::Value::Table(table));

        toml::Value::Table(toml)

    }

    fn decode_from_toml(&mut self, value: BTreeMap<String, toml::Value>) {

        self.aliases.clear();

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

        self.greetings.clear();

        if let Some(&toml::Value::Table(ref table)) = value.get("greetings") {
            for (nickname, effect) in table {
                if let toml::Value::String(ref effect) = *effect {
                    self.greetings.insert(nickname.clone(), effect.clone());
                }
            }
        }

        self.admins = from_toml_strings(value.get("admins"));
        self.uploaders = from_toml_strings(value.get("uploaders"));
        self.banned = from_toml_strings(value.get("banned"));

    }

}

// Helpers --------------------------------------------------------------------
fn to_toml_strings(items: &[String]) -> toml::Value {
    toml::Value::Array(items.iter().map(|item| {
        toml::Value::String(item.to_string())

    }).collect())
}

fn from_toml_strings(array: Option<&toml::Value>) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(&toml::Value::Array(ref array)) = array {
        for item in array {
            if let toml::Value::String(ref item) = *item {
                items.push(item.clone());
            }
        }
    }
    items
}

