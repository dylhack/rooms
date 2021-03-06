use crate::bot::core;
use crate::config::{Config, Serving};
use log::info;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Handler;

fn skip_review(new: &VoiceState, old_opt: &Option<VoiceState>) -> bool {
    if old_opt.is_none() {
        return false;
    }
    let old = old_opt.as_ref().unwrap();

    return new.channel_id == old.channel_id;
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, rdy: Ready) {
        let perms = Permissions::from_bits(268438592).unwrap();
        let us = &rdy.user;
        let guilds = &rdy.guilds;
        info!(
            "
Ready as {}
 * Serving {} guilds
 * Invite URL: {}",
            us.tag(),
            guilds.len(),
            us.invite_url(&ctx, perms).await.unwrap(),
        );
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        opt_guild_id: Option<GuildId>,
        opt_old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if (skip_review(&new, &opt_old)) {
            return;
        }

        let serving: Serving;
        {
            let data = ctx.data.read().await;
            let config = data.get::<Config>().expect("Failed to retrieve config");

            match opt_guild_id {
                Some(guild_id) => {
                    if let Some(_serving) = config.serving.get(guild_id.as_u64()) {
                        serving = _serving.clone();
                    } else {
                        return;
                    }
                }
                None => return,
            }
        }

        // Review the voice channel they left
        if let Some(old) = opt_old {
            let ctx_clone = ctx.clone();
            let serve_clone = serving.clone();
            core::review_state(&ctx_clone, &serve_clone, &old).await;
        }
        // Review the voice channel they joined
        core::review_state(&ctx, &serving, &new).await;
    }
}
