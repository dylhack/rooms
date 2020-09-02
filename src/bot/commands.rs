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
    return perms.manage_channels();
}

#[command]
fn link(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write();
    let mut config = data.get::<Config>().unwrap();
    Ok(())
}

#[command]
fn unlink(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write();
    let mut config = data.get::<Config>().unwrap();
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

    Ok(())
}
