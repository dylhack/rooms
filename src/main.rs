use crate::config::Config;
use simple_logger::SimpleLogger;
use log::{warn};

mod bot;
mod config;

#[tokio::main]
async fn main() {
	SimpleLogger::new().init().unwrap();
    let config = Config::new("./config.yml".to_string());

    if config.token.is_empty() {
        warn!("Please fill out the config.yml");
        return;
    }

    bot::start(config).await;
}
