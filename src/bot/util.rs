use serenity::framework::standard::Args;
use serenity::client::Context;
use crate::config::Room;
use std::sync::Arc;
use serenity::prelude::RwLock;
use serenity::model::prelude::*;


// Get the channels a user might be talking about in a message.
// args can be [<#channel id>, channel id] or reversed
pub fn parse_channels(ctx: &Context, msg: &Message, args: &mut Args) -> Option<(Channel, Channel)> {
    let text_id;
    let mut voice_id = ChannelId(0);

    if let Some(channels) = &msg.mention_channels {
        if channels.is_empty() {
            return None;
        }
        text_id = channels[0].id;
    } else {
        return None;
    }

    for _arg in args.iter::<String>() {
        match _arg {
            Err(_) => return None,
            Ok(arg) => {
                if let Ok(channel_id) = arg.parse::<u64>() {
                    voice_id = ChannelId(channel_id);
                }
            },
        }
    };

    let text;
    let voice;

    match text_id.to_channel(ctx) {
        Ok(chan) => text = chan,
        Err(_) => return None,
    }
    match voice_id.to_channel(ctx) {
        Ok(chan) => voice = chan,
        Err(_) => return None,
    }

    return Some((voice, text));
}

pub fn respond(ctx: &Context, msg: &Message, body: &String) {
    let res = format!("<@{}>, {}", msg.author.id, body);
    if let Err(why) = msg.channel_id.say(&ctx, &res) {
        println!("Failed to send a message in #{} because\n{}", msg.channel_id, why);
    }
}

// good reacts to a message when a user used a command correctly
pub fn good(ctx: &Context, msg: &Message) {
    react(ctx, msg, "".to_string())
}

// bad reacts to a message when a user used a command incorrectly
pub fn bad(ctx: &Context, msg: &Message) {
    react(ctx, msg, "".to_string())
}

fn react(ctx: &Context, msg: &Message, unicode: String) {
    if let Err(why) = msg.react(ctx, ReactionType::Unicode(unicode)) {
        println!("Failed to react to {} because\n{}", msg.author.id, why);
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
