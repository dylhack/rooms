use crate::bot::core;
use crate::config::{Config, Serving};
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, rdy: Ready) {
        let us = &rdy.user;

        println!("Ready as {}#{}", us.name, us.discriminator);
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        opt_guild_id: Option<GuildId>,
        opt_old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("Failed to retrieve config");
        let serving: &Serving;

        match opt_guild_id {
            Some(guild_id) => {
                if let Some(_serving) = config.serving.get(guild_id.as_u64()) {
                    serving = _serving;
                } else {
                    return;
                }
            }
            None => return,
        }

        // Review the voice channel they left
        if let Some(old) = opt_old {
            core::review_state(&ctx, &serving, &old).await;
        }
        // Review the voice channel they're joining
        core::review_state(&ctx, &serving, &new).await;
    }
}
