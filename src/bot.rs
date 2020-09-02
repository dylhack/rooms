use crate::bot::events::Handler;
use crate::config::Config;
use serenity::client::Client;
use serenity::framework::StandardFramework;


mod core;
mod commands;
mod events;
mod util;

impl typemap::Key for Config {
    type Value = Config;
}

pub fn start(config: Config) {
    let mut client = Client::new(&config.token, Handler).expect("Failed to create new client.");

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(&config.prefix);
                c.allow_dm(false);
                c.case_insensitivity(true);
                return c;
            })
            .group(&commands::ADMINCOMMANDS_GROUP)
            .group(&commands::COMMANDS_GROUP),
    );

    {
        let mut data = client.data.write();
        data.insert::<Config>(config);
    }

    if let Err(why) = client.start() {
        panic!("Failed to start bot because\n{}", why);
    }
}

