use crate::config::Room;
use crate::bot::util::grant_access;
use serenity::client::Context;
use crate::bot::util::get_channels;
use serenity::model::prelude::*;

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
