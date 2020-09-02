use crate::config::Config;


mod bot;
mod config;

#[tokio::main]
async fn main() {
    let config = Config::new("./config.yml".to_string());
    bot::start(config).await;
}
