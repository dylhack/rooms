use crate::config::Room;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::prelude::*;

// Get the channels a user might be talking about in a message.
// args can be [<#channel id>, channel id] or reversed
pub async fn parse_channels(ctx: &Context, args: &mut Args) -> Option<(Channel, Channel)> {
    let mut text_id = ChannelId(0);
    let mut voice_id = ChannelId(0);

    for _arg in args.iter::<String>() {
        match _arg {
            Err(_) => return None,
            Ok(arg) => {
                if let Ok(channel_id) = arg.parse::<u64>() {
                    voice_id = ChannelId(channel_id);
                } else if arg.starts_with("<#") {
                    if let Some(extract) = arg.get(2..arg.len() - 1) {
                        if let Ok(channel_id) = extract.parse::<u64>() {
                            text_id = ChannelId(channel_id);
                        }
                    }
                }
            }
        }
    }

    let text;
    let voice;

    match text_id.to_channel(ctx).await {
        Ok(chan) => text = chan,
        Err(_) => return None,
    }
    match voice_id.to_channel(ctx).await {
        Ok(chan) => voice = chan,
        Err(_) => return None,
    }

    return Some((voice, text));
}

pub async fn respond(ctx: &Context, msg: &Message, body: &String) {
    let res = format!("<@{}>, {}", msg.author.id, body);
    if let Err(why) = msg.channel_id.say(&ctx, &res).await {
        println!(
            "Failed to send a message in #{} because\n{}",
            msg.channel_id, why
        );
    }
}

// good reacts to a message when a user used a command correctly
pub async fn good(ctx: &Context, msg: &Message) {
    react(ctx, msg, "✅".to_string()).await
}

// bad reacts to a message when a user used a command incorrectly
pub async fn bad(ctx: &Context, msg: &Message) {
    react(ctx, msg, "❌".to_string()).await
}

pub async fn warn(ctx: &Context, msg: &Message) {
    react(ctx, msg, "⚠️".to_string()).await
}

async fn react(ctx: &Context, msg: &Message, unicode: String) {
    if let Err(why) = msg.react(ctx, ReactionType::Unicode(unicode)).await {
        println!("Failed to react to {} because\n{}", msg.author.id, why);
    }
}

pub async fn get_channels(ctx: &Context, room: &Room) -> Option<(GuildChannel, GuildChannel)> {
    let mut channel_rw;

    if let Ok(_channel) = room.voice_id.to_channel(ctx).await {
        if let Some(_guild_rw) = _channel.guild() {
            channel_rw = _guild_rw;
        } else {
            return None;
        }
    } else {
        return None;
    }

    let voice_channel = channel_rw;

    if let Ok(_channel) = room.text_id.to_channel(ctx).await {
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

pub async fn grant_access(ctx: &Context, text: &GuildChannel, member_id: UserId) {
    let overwrite = PermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(member_id),
    };

    manage_access(ctx, text, &overwrite, member_id).await;
    println!("Granted access for {} in {}", member_id, text.id);
}

pub async fn revoke_access(ctx: &Context, text: &GuildChannel, member_id: UserId) {
    let overwrite = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Member(member_id),
    };

    manage_access(ctx, text, &overwrite, member_id).await;
    println!("Revoked access for {} in {}", member_id, text.id);
}

async fn manage_access(
    ctx: &Context,
    text: &GuildChannel,
    overwrite: &PermissionOverwrite,
    member_id: UserId,
) {
    if let Err(why) = text.create_permission(ctx, overwrite).await {
        println!("Failed to grant {} access because\n{}", member_id, why);
    }
}
