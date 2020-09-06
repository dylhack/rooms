use crate::config::Config;
use log::{info, warn, LevelFilter};
use simple_logger::SimpleLogger;

mod bot;
mod config;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    info!("Starting...");
    let config = Config::new();

    if config.token.is_empty() {
        warn!("Please fill out the config.yml");
        return;
    }

    bot::start(config).await;
}
