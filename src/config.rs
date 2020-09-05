use log::info;
use serde::{Deserialize, Serialize};
use serde_yaml;
use serenity::model::id::GuildId;
use serenity::model::prelude::ChannelId;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// For getting a custom config path
const ENV_VAR: &'static str = "CONFIG_PATH";
const DEFAULT_LOCATION: &'static str = "./config.yml";

// Serving represents a guild the bot is serving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Serving {
    pub guild_id: GuildId,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub voice_id: ChannelId,
    pub text_id: ChannelId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub prefix: String,
    pub serving: BTreeMap<u64, Serving>,
}

impl Config {
    pub fn new() -> Config {
        let location = env::var(ENV_VAR).unwrap_or(DEFAULT_LOCATION.to_string());
        match Config::retrieve() {
            Some(conf) => conf,
            None => {
                let conf = Config {
                    token: String::new(),
                    prefix: String::from("!"),
                    serving: BTreeMap::new(),
                };
                conf.save();
                info!("Created a new config at {}", &location);
                return conf;
            }
        }
    }

    pub fn save(&self) {
        let location = env::var(ENV_VAR).unwrap_or(DEFAULT_LOCATION.to_string());
        let serialized = serde_yaml::to_string(&self).expect("Failed to serialize config.");
        match File::create(&location) {
            Ok(mut file) => {
                file.write_all(serialized.as_bytes())
                    .expect("Failed to write to config");
            }
            Err(_) => {
                panic!("Failed to save config to {}", &location);
            }
        }
    }

    fn retrieve() -> Option<Config> {
        let location = env::var(ENV_VAR).unwrap_or(DEFAULT_LOCATION.to_string());
        match File::open(location) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Err(_) = file.read_to_string(&mut contents) {
                    return None;
                };

                match serde_yaml::from_str(&contents) {
                    Ok(deserialized) => Some(deserialized),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}
