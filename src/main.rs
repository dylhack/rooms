use crate::config::Config;
use simple_logger::SimpleLogger;
use log::{LevelFilter, info, warn};

mod bot;
mod config;

#[tokio::main]
async fn main() {
	SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("serenity", LevelFilter::Off)
        .with_module_level("reqwest", LevelFilter::Off)
        .with_module_level("rustls", LevelFilter::Off)
        .with_module_level("hyper", LevelFilter::Off)
        .with_module_level("async_tungstenite", LevelFilter::Off)
        .with_module_level("tungstenite", LevelFilter::Off)
		.init()
		.unwrap();
	info!("Starting...");
    let config = Config::new("./config.yml".to_string());

    if config.token.is_empty() {
        warn!("Please fill out the config.yml");
        return;
    }

    bot::start(config).await;
}
