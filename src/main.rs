use crate::config::Config;

mod bot;
mod config;

#[tokio::main]
async fn main() {
    let config = Config::new("./config.yml".to_string());

    if config.token.is_empty() {
        println!("Please fill out the config.yml");
        return;
    }

    bot::start(config).await;
}
