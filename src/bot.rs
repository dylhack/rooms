use crate::bot::events::Handler;
use crate::config::Config;
use crate::config::Room;
use serenity::client::Client;
use serenity::client::Context;
use serenity::framework::StandardFramework;
use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
use serenity::model::prelude::*;
use serenity::prelude::RwLock;
use std::sync::Arc;

mod commands;
mod events;

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

pub fn review(ctx: &Context, room: &Room) {
    let channels = get_channels(ctx, room);
    let voice_rw;
    let text_rw;

    match channels {
        Some((_voice, _text)) => {
            voice_rw = _voice;
            text_rw = _text;
        }
        None => return,
    }

    let voice = voice_rw.read();
    let text = text_rw.read();
    let mut members: Vec<Member>;

    match voice.members(ctx) {
        Ok(_members) => members = _members,
        Err(_) => return,
    }

    let mut i = 0;
    while i != members.len() {
        let member = &members[i].clone();
        let user = &member.user.read();
        if let Ok(perms) = text.permissions_for_user(&ctx, &user.id) {
            if !perms.read_messages() {
                grant_access(&ctx, &text, user.id);
                members.remove(i);
            }
        }

        i += 1;
    }

    i = 0;
    while i != members.len() {
        i += 1;
        let member = &members[i].clone();
        let user = &member.user.read();
        if let Err(why) = text.delete_permission(ctx, PermissionOverwriteType::Member(user.id)) {
            println!("Failed to revoke {} access because\n{}", user.id, why);
        }
    }
}

fn get_channels(
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

fn grant_access(ctx: &Context, text: &GuildChannel, member_id: UserId) {
    let overwrite = PermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(member_id),
    };

    if let Err(why) = text.create_permission(ctx, &overwrite) {
        println!("Failed to grant {} access because\n{}", member_id, why);
    }
}
