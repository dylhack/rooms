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
// auth checks if someone has the right perms to run the link and unlink commands. The current
// required permissions are: Manage Channels (bits: 16)
async fn auth(ctx: &Context, msg: &Message) -> CheckResult {
    let guild_id;

    // log reports when a user has the required permissions.
    let log = |msg: &Message| {
        let user = &msg.author;
        info!(
            "
Command Execution
 * User: {}
 * Command: {}
 * Link: {}",
            user.tag(),
            msg.content,
            msg.link(),
        );
    };

    // fail_log reports when a user doesn't have the require permissions.
    let fail_log = |msg: &Message, reason: &String| {
        let user = &msg.author;
        warn!(
            "
Failed Command Execution
 * User: {}
 * Command: {}
 * Link: {}
 * Reason: {}",
            user.tag(),
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

    // Check if they have the required permissions. See check_perms
    // to see what permissions are needed
    let mut perms = guild.member_permissions(msg.author.id);

    if check_perms(&perms) {
        log(&msg);
        return Success;
    }

    // check if they have "manage channel" in the channel that they're executing the command.
    perms = guild.user_permissions_in(msg.channel_id, msg.author.id);

    if check_perms(&perms) {
        log(&msg);
        return Success;
    }

    let reason = "You miss the required permissions to run this command.".to_string();
    fail_log(&msg, &reason);
    return Failure(User(reason));
}

// check_perms compliments auth. It makes sure that the user running a command has administrator and
// or "manage channels" (bits: 16)
fn check_perms(perms: &Permissions) -> bool {
    return perms.administrator() || perms.manage_channels();
}

#[command]
// link allows users to link a text-channel and voice-channel together. When a voice and text
// channel are linked together it's called a "Room" and every guild has it's own vector of rooms
// stored in the config.
// args = [#text-channel, voice channel ID] or [voice channel ID, #text-channel]
async fn link(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut config = data.get_mut::<Config>().unwrap().clone();
    let mut serving;

    // Get the vector of rooms for this guild.
    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        // if they don't have on then make one.
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

    // parse_channels will get us the channels the user is referring to in their message.
    let channels = util::parse_channels(ctx, &mut args).await;
    let text;
    let voice;

    match channels {
        Some((_voice, _text)) => {
            voice = _voice;
            text = _text;
        }
        // If parse_channels didn't return anything then the user didn't provide two channels or
        // mistakenly two of the same type of channel.
        None => {
            let res = "Please mention a text channel and ID of the voice channel.".to_string();
            util::bad(ctx, msg).await;
            util::respond(ctx, msg, &res).await;
            return Ok(());
        }
    }

    // Iterate through the guild's rooms and make sure the channels they provided aren't already
    // linked with something else.
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

    // Finally link the channels together and establish a room.
    let room = Room {
        voice_id: voice.id(),
        text_id: text.id(),
    };

    // Save it to the config
    serving.rooms.push(room);
    config.serving.insert(*serving.guild_id.as_u64(), serving);
    config.save();
    data.insert::<Config>(config.clone());
    // React to their message to let them know everything went right.
    util::good(ctx, msg).await;
    Ok(())
}

#[command]
// unlink will remove a link between a text-channel and voice-channel
// args can be a vector of #text-channels, voice channel IDs, or a combination. It
// will unlinked all the channels provided (so essentially you can chain the channels.)
async fn unlink(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut serving;
    let mut config;
    {
        let mut data = ctx.data.write().await;
        config = data.get_mut::<Config>().unwrap().clone();
    }

    // Get all the rooms for the guild.
    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        util::good(ctx, msg).await;
        return Ok(());
    }

    // channel_ids are all the channels the user is talking about unlinking.
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

    // List all the channels successfully unlinked
    let mut unlinked = String::new();
    // List all the channels failed to unlink
    let mut not_unlinked = String::new();

    // Go through all the channels and unlink them.
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

    // After all the channels are unlinked save it back into the config.
    {
        let mut data = ctx.data.write().await;
        config.serving.insert(*serving.guild_id.as_u64(), serving);
        config.save();
        data.insert::<Config>(config.clone());
    }

    // If all the channels were unlinked successfully
    if !unlinked.is_empty() && not_unlinked.is_empty() {
        util::respond(ctx, msg, &unlinked).await;
        util::good(ctx, msg).await;

    // If none of the channels they gave were unlinked successfully
    } else if unlinked.is_empty() && !not_unlinked.is_empty() {
        util::respond(ctx, msg, &not_unlinked).await;
        util::bad(ctx, msg).await;

    // else if there were some unlinekd and some not then give them a warning.
    } else if !unlinked.is_empty() && !not_unlinked.is_empty() {
        util::respond(ctx, msg, &unlinked).await;
        util::respond(ctx, msg, &not_unlinked).await;
        util::warn(ctx, msg).await;
    }

    Ok(())
}

#[command]
// list will send a message with all the channels that are linked with each other.
// output example:
// Linked Channels:
// - <#text-channel ID> -> voice channel name
// - <#text-channel ID> -> voice channel name
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<Config>().unwrap();
    let serving;

    // Get the rooms for this guild
    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s;
    } else {
        // if they don't have any rooms then tell them there are no channels linked.
        util::respond(&ctx, &msg, &"This server has no rooms".to_string()).await;
        util::good(ctx, msg).await;
        return Ok(());
    }

    // This is what's responded
    let mut list = String::from("Linked Channels:\n");

    // Iterate through all the rooms and list them
    for room in serving.rooms.iter() {
        let mut list_item: String;
        match room.text_id.to_channel(ctx).await {
            Ok(text) => {
                if let Some(_) = text.guild() {
                    list_item = format!(" - <#{}> -> ", &room.text_id);
                } else {
                    list_item = format!(" - {} -> ", room.text_id);
                }
            }
            Err(_) => {
                list_item = format!(" - {} -> ", room.text_id);
            }
        }

        match room.voice_id.to_channel(ctx).await {
            Ok(voice) => {
                let name_or_id;
                if let Some(guild_chan) = voice.guild() {
                    name_or_id = guild_chan.name;
                } else {
                    name_or_id = room.voice_id.to_string();
                }
                list_item += &format!("{}\n", name_or_id).to_string();
            }
            Err(_) => {
                list_item += &format!("{}\n", &room.voice_id).to_string();
            }
        }

        list.push_str(list_item.as_str());
    }

    util::respond(&ctx, &msg, &list).await;
    util::good(ctx, msg).await;
    Ok(())
}
