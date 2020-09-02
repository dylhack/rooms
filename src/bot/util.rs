use serenity::client::Context;
use crate::config::Room;
use std::sync::Arc;
use serenity::prelude::RwLock;
use serenity::model::prelude::*;


pub fn respond(ctx: &Context, msg: &Message, body: &String) {
    let res = format!("<@{}>, {}", msg.author.id, body);
    if let Err(why) = msg.channel_id.say(&ctx, &res) {
        println!("Failed to send a message in #{} because\n{}", msg.channel_id, why);
    }
}

pub fn get_channels(
    ctx: &Context,
    room: &Room,
) -> Option<(Arc<RwLock<GuildChannel>>, Arc<RwLock<GuildChannel>>)> {
    let mut channel_rw;

    if let Ok(_channel) = room.voice_id.to_channel(ctx) {
        if let Some(_guild_rw) = _channel.guild() {
            channel_rw = _guild_rw;
        } else {
            return None;
        }
    } else {
        return None;
    }

    let voice_channel = channel_rw;

    if let Ok(_channel) = room.text_id.to_channel(ctx) {
        if let Some(_guild_rw) = _channel.guild() {
            channel_rw = _guild_rw;
        } else {
            return None;
        }
    } else {
        return None;
    }

    let text_channel = channel_rw;
    Some((voice_channel, text_channel))
}

pub fn grant_access(ctx: &Context, text: &GuildChannel, member_id: UserId) {
    let overwrite = PermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(member_id),
    };

    if let Err(why) = text.create_permission(ctx, &overwrite) {
        println!("Failed to grant {} access because\n{}", member_id, why);
    }
}
