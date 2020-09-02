use crate::config::Room;
use serenity::framework::standard::CommandError;
use crate::config::Serving;
use crate::config::Config;
use crate::bot::util;
use serenity::client::Context;
use serenity::framework::standard::macros::{check, command, group};
use serenity::framework::standard::CheckResult::*;
use serenity::framework::standard::Reason::User;
use serenity::framework::standard::*;
use serenity::model::channel::Message;
use serenity::model::Permissions;

#[group()]
#[commands(link, unlink)]
#[checks(auth)]
pub struct AdminCommands;

#[group()]
#[commands(list)]
pub struct Commands;

#[check()]
#[name("auth")]
fn auth(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    let guild_id;

    if let Some(_guild_id) = msg.guild_id {
        guild_id = _guild_id;
    } else {
        return Failure(User(
            "This command needs to be executed in a guild.".to_string(),
        ));
    }

    let guild_rw;
    if let Some(_guild) = guild_id.to_guild_cached(ctx) {
        guild_rw = _guild;
    } else {
        return Failure(User("Failed to fetch this guild in cache.".to_string()));
    }
    let guild = guild_rw.read();
    let mut perms = guild.member_permissions(msg.author.id);

    if check_perms(&perms) {
        return Success;
    }

    perms = guild.user_permissions_in(msg.channel_id, msg.author.id);

    if check_perms(&perms) {
        return Success;
    }

    return Failure(User(
        "You miss the required permissions to run this command.".to_string(),
    ));
}

fn check_perms(perms: &Permissions) -> bool {
    return perms.administrator() || perms.manage_channels();
}

#[command]
fn link(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let config = data.get_mut::<Config>().unwrap();
    let mut serving;

    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        if let Some(guild_id) = msg.guild_id {
            serving = Serving{
                guild_id,
                rooms: Vec::<Room>::new(),
            };
        } else {
            util::bad(ctx, msg);
            return Err(CommandError("Please use this command in a guild.".to_string()));
        }
    }

    let channels = util::parse_channels(ctx, msg, &mut args);
    let text;
    let voice;


    match channels {
        Some((_voice, _text)) => {
            voice = _voice; 
            text = _text;
        },
        None => {
            util::bad(ctx, msg);
            return Err(CommandError("Please mention a text channel and ID of the voice channel.".to_string()))
        }
    }

    for room in serving.rooms.iter() {
        if room.voice_id == voice.id() || room.text_id == text.id() {
            util::bad(ctx, msg);
            if room.voice_id == voice.id() {
                return Err(CommandError("That voice channel is already linked with something.".to_string()))
            } else {
                return Err(CommandError("That text channel is already linked with something.".to_string()))
            }
        }
    }

    let mut update = |serving: Serving| {
        let mut data = ctx.data.write();
        config.serving.insert(*serving.guild_id.as_u64(), serving);
        data.insert::<Config>(config.clone());
    };

    let room = Room{
        voice_id: voice.id(),
        text_id: text.id(),
    };

    serving.rooms.push(room);
    update(serving);
    Ok(())
}

#[command]
// Args = #channel or channel ID
fn unlink(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut serving;
    let mut data = ctx.data.write();
    let config = data.get_mut::<Config>().unwrap();


    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s.clone();
    } else {
        util::good(ctx, msg);
        return Ok(());
    }

    let mut update = |serving: Serving| {
        let mut data = ctx.data.write();
        config.serving.insert(*serving.guild_id.as_u64(), serving);
        data.insert::<Config>(config.clone());
    };


    if let Some(channels) = &msg.mention_channels {
        if !channels.is_empty() {
            let mut unlinked = String::from("Unlinked: \n");
            for channel in channels.iter() {
                let mut i = 0;
                for room in serving.rooms.clone().iter() {
                    if room.text_id.as_u64() == channel.id.as_u64() {
                        serving.rooms.remove(i);
                        unlinked.push_str(format!(" - <#{}>\n", channel.id).as_str());
                        continue;
                    }
                    i += 1;
                }
            }
            util::respond(ctx, msg, &unlinked);
            util::good(ctx, msg);
            update(serving);
            
            return Ok(());
        }
    }



    Ok(())
}

#[command]
fn list(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();
    let config = data.get::<Config>().unwrap();
    let serving;

    if let Some(_s) = config.serving.get(msg.guild_id.unwrap().as_u64()) {
        serving = _s;
    } else {
        util::respond(&ctx, &msg, &"This server doesn't have any channels linked.".to_string());
        util::good(ctx, msg);
        return Ok(());
    }

    let mut list = String::from("Linked Channels:\n");

    for room in serving.rooms.iter() {
        let with_name = |name: &String| -> String {
            return format!(" - <#{}> -> {}\n", 
                room.text_id.as_u64(), 
                name,
            );
        };

        let without_name = || -> String {
            return format!(" - <#{}> -> <#{}>\n", 
                room.text_id.as_u64(), 
                room.voice_id.as_u64(),
            );
        };

        let list_item: String;

        match room.voice_id.to_channel(&ctx) {
            Ok(voice) => {
                if let Some(guild_chan) = voice.guild() {
                    list_item = with_name(&guild_chan.read().name);
                } else {
                    list_item = without_name();
                }
            },
            Err(_) => {
                list_item = without_name();
            }
        }

        list.push_str(list_item.as_str());
    }

    util::respond(&ctx, &msg, &list);
    util::good(ctx, msg);
    Ok(())
}
