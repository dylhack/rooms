use crate::bot::events::Handler;
use crate::config::Config;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use serenity::prelude::TypeMapKey;

mod commands;
mod core;
mod events;
mod util;

impl TypeMapKey for Config {
    type Value = Config;
}

pub async fn start(config: Config) {
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(&config.prefix);
            c.allow_dm(false);
            c.case_insensitivity(true);
            return c;
        })
        .group(&commands::ADMINCOMMANDS_GROUP)
        .group(&commands::COMMANDS_GROUP);
    let mut client = Client::new(&config.token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    {
        let mut data = client.data.write().await;
        data.insert::<Config>(config);
    }

    if let Err(why) = client.start().await {
        panic!("Failed to start bot because\n{}", why);
    }
}
