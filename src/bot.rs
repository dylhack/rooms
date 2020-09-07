mod commands;
mod core;
mod events;
mod util;

use crate::bot::events::Handler;
use crate::config::Config;
use log::warn;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use serenity::prelude::TypeMapKey;

impl TypeMapKey for Config {
    type Value = Config;
}

pub async fn start(config: Config) {
    let framework = StandardFramework::new()
        .group(&commands::ADMINCOMMANDS_GROUP)
        .group(&commands::COMMANDS_GROUP)
        .configure(|c| {
            c.prefix(&config.prefix);
            c.allow_dm(false);
            c.case_insensitivity(true);
            return c;
        });

    let mut client = Client::new(&config.token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    {
        let mut data = client.data.write().await;
        data.insert::<Config>(config);
    }

    if let Err(e) = client.start().await {
        warn!("Failed to login, is the token correct?\n{}", e);
    }
}
