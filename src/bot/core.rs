use crate::bot::util::get_channels;
use crate::bot::util::grant_access;
use crate::config::Room;
use serenity::client::Context;
use serenity::model::prelude::*;

pub async fn review(ctx: &Context, room: &Room) {
    let channels = get_channels(ctx, room).await;
    let voice;
    let text;

    match channels {
        Some((_voice, _text)) => {
            voice = _voice;
            text = _text;
        }
        None => return,
    }

    let mut members: Vec<Member>;

    match voice.members(&ctx).await {
        Ok(_members) => members = _members,
        Err(_) => return,
    }

    let mut i = 0;
    while i != members.len() {
        let member = &members[i].clone();
        let user = &member.user;
        if let Ok(perms) = text.permissions_for_user(&ctx, &user.id).await {
            if !perms.read_messages() {
                grant_access(&ctx, &text, user.id).await;
                members.remove(i);
            }
        }

        i += 1;
    }

    i = 0;
    while i != members.len() {
        i += 1;
        let member = &members[i].clone();
        let user = &member.user;
        if let Err(why) = text.delete_permission(ctx, PermissionOverwriteType::Member(user.id)).await {
            println!("Failed to revoke {} access because\n{}", user.id, why);
        }
    }
}
