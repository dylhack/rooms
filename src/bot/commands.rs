use crate::bot::util;
use crate::config::{Config, Room, Serving};
use log::{info, warn};
use serenity::client::Context;
use serenity::framework::standard::macros::{check, command, group};
use serenity::framework::standard::CheckResult::{Failure, Success};
use serenity::framework::standard::Reason::User;
use serenity::framework::standard::*;
use serenity::model::prelude::*;

#[group()]
#[commands(link, unlink)]
#[checks(auth)]
pub struct AdminCommands;

#[group()]
#[commands(list)]
pub struct Commands;

#[check()]
#[name("auth")]
async fn auth(ctx: &Context, msg: &Message) -> CheckResult {
    let guild_id;
    
    let log = |msg: &Message| {
        let user = &msg.author;
        info!(
            "Command Execution\n
            User: {}#{}\n
            Command: {}\n
            Link: {}", 
            user.name, user.discriminator,
            msg.content, 
            msg.link(),
        );
    };

    let fail_log = |msg: &Message, reason: &String| {
        let user = &msg.author;
        warn!(
            "Failed Command Execution\n
            User: {}#{}\n
            Command: {}\n
            Link: {}\n
            Reason: {}", 
            user.name, user.discriminator, 
            msg.content,
            msg.link(),
            reason,
        );
    };

    // Make sure they're executing the command in a guild.
    if let Some(_guild_id) = msg.guild_id {
        guild_id = _guild_id;
    } else {
        let reason = "This command needs to be executed in a guild.".to_string();
        fail_log(&msg, &reason);
        return Failure(User(reason));
    }

    let guild;
    if let Some(_guild) = guild_id.to_guild_cached(ctx).await {
        guild = _guild;
    } else {
        let reason = "Failed to fetch this guild in cache.".to_string();
        fail_log(&msg, &reason);
        return Failure(User(reason));
    }

    // Check if they have the require perms. See check_perms to see what permissions are needed   
    let mut perms = guild.member_permissions(msg.author.id);


    if check_perms(&perms) {
        log(&msg);
        return Success;
    }

    perms = guild.user_permissions_in(msg.channel_id, msg.author.id);

    if check_perms(&perms) {
        log(&msg);
        return Success;
    }

    let reason = "You miss the required permissions to run this command.".to_string();
    fail_log(&msg, &reason);
    return Failure(User(reason));
}

fn check_perms(perms: &Permissions) -> bool {
    return perms.administrator() || perms.manage_channels();
}

#[command]
async fn link(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut config = data.get_mut::<Config>().unwrap().clone();
    let mut serving;

    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        if let Some(guild_id) = msg.guild_id {
            serving = Serving {
                guild_id,
                rooms: Vec::<Room>::new(),
            };
        } else {
            let res = "Please use this command in a guild.".to_string();
            util::bad(ctx, msg).await;
            util::respond(ctx, msg, &res).await;
            return Ok(());
        }
    }

    let channels = util::parse_channels(ctx, &mut args).await;
    let text;
    let voice;

    match channels {
        Some((_voice, _text)) => {
            voice = _voice;
            text = _text;
        }
        None => {
            let res = "Please mention a text channel and ID of the voice channel.".to_string();
            util::bad(ctx, msg).await;
            util::respond(ctx, msg, &res).await;
            return Ok(());
        }
    }

    for room in serving.rooms.iter() {
        if room.voice_id == voice.id() || room.text_id == text.id() {
            util::bad(ctx, msg).await;
            let res;
            if room.voice_id == voice.id() {
                res = "That voice channel is already linked with something.".to_string();
            } else {
                res = "That text channel is already linked with something.".to_string();
            }
            util::respond(ctx, msg, &res).await;
            return Ok(());
        }
    }

    let room = Room {
        voice_id: voice.id(),
        text_id: text.id(),
    };

    serving.rooms.push(room);
    config.serving.insert(*serving.guild_id.as_u64(), serving);
    config.save();
    data.insert::<Config>(config.clone());
    util::good(ctx, msg).await;
    Ok(())
}

// Args = #channel or channel ID
#[command]
async fn unlink(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut serving;
    let mut config;
    {
        let mut data = ctx.data.write().await;
        config = data.get_mut::<Config>().unwrap().clone();
    }

    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        util::good(ctx, msg).await;
        return Ok(());
    }

    let mut channel_ids = Vec::<ChannelId>::new();
    for _arg in args.iter::<String>() {
        match _arg {
            Ok(arg) => {
                if arg.starts_with("<#") {
                    if let Some(extract) = arg.get(2..arg.len() - 1) {
                        if let Ok(channel_id) = extract.parse::<u64>() {
                            channel_ids.push(ChannelId(channel_id));
                        }
                    }
                } else if let Ok(channel_id) = arg.parse::<u64>() {
                    channel_ids.push(ChannelId(channel_id));
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    let mut unlinked = String::new();
    let mut not_unlinked = String::new();

    for channel_id in channel_ids {
        let mut i = 0;
        match channel_id.to_channel(ctx).await {
            Ok(channel) => {
                for room in serving.rooms.clone().iter() {
                    if room.text_id == channel.id() {
                        if unlinked.is_empty() {
                            unlinked.push_str("Unlinked: \n");
                        }

                        serving.rooms.remove(i);
                        unlinked.push_str(format!(" - <#{}>\n", channel.id()).as_str());
                        break;
                    }
                    i += 1;
                }
            }
            Err(_) => {
                if not_unlinked.is_empty() {
                    not_unlinked.push_str("Couldn't Find: \n");
                }
                not_unlinked.push_str(format!(" - <#{}>", channel_id).as_str());
            }
        }
    }

    {
        let mut data = ctx.data.write().await;
        config.serving.insert(*serving.guild_id.as_u64(), serving);
        config.save();
        data.insert::<Config>(config.clone());
    }

    if !unlinked.is_empty() && not_unlinked.is_empty() {
        util::respond(ctx, msg, &unlinked).await;
        util::good(ctx, msg).await;
    } else if unlinked.is_empty() && !not_unlinked.is_empty() {
        util::respond(ctx, msg, &not_unlinked).await;
        util::bad(ctx, msg).await;
    } else if !unlinked.is_empty() && !not_unlinked.is_empty() {
        util::respond(ctx, msg, &unlinked).await;
        util::respond(ctx, msg, &not_unlinked).await;
        util::warn(ctx, msg).await;
    }

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<Config>().unwrap();
    let serving;

    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s;
    } else {
        util::respond(
            &ctx,
            &msg,
            &"This server doesn't have any channels linked.".to_string(),
        )
        .await;
        util::good(ctx, msg).await;
        return Ok(());
    }

    let mut list = String::from("Linked Channels:\n");

    for room in serving.rooms.iter() {
        let with_name = |name: &String| -> String {
            return format!(" - <#{}> -> {}\n", room.text_id.as_u64(), name,);
        };

        let without_name = || -> String {
            return format!(
                " - <#{}> -> <#{}>\n",
                room.text_id.as_u64(),
                room.voice_id.as_u64(),
            );
        };

        let list_item: String;

        match room.voice_id.to_channel(ctx).await {
            Ok(voice) => {
                if let Some(guild_chan) = voice.guild() {
                    list_item = with_name(&guild_chan.name);
                } else {
                    list_item = without_name();
                }
            }
            Err(_) => {
                list_item = without_name();
            }
        }

        list.push_str(list_item.as_str());
    }

    util::respond(&ctx, &msg, &list).await;
    util::good(ctx, msg).await;
    Ok(())
}
