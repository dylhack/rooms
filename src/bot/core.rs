use crate::bot::util::{get_channels, grant_access, revoke_access};
use crate::config::{Room, Serving};
use serenity::client::Context;
use serenity::model::prelude::*;

pub async fn review_state(ctx: &Context, serving: &Serving, state: &VoiceState) {
    if let Some(channel_id) = state.channel_id {
        if let Some(room) = get_room(&serving, &channel_id) {
            review(&ctx, &room).await;
        }
    }
}

async fn review(ctx: &Context, room: &Room) {
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

    let mut members_in_vc: Vec<Member>;

    match voice.members(&ctx).await {
        Ok(_members) => members_in_vc = _members,
        Err(_) => return,
    }

    for perm in &text.permission_overwrites {
        match perm.kind {
            PermissionOverwriteType::Member(user_id) => {
                let (is_in_vc, i) = in_vc(user_id, &members_in_vc);

                // Remove them from the text channel if they're not
                // in the voice channel.
                if !is_in_vc {
                    revoke_access(ctx, &text, user_id).await;

                // Otherwise if they're in the vc and have access to the
                // text-channel then remove them from the vec. The vec
                // will be iterated through later and add the remaining
                // members that can't see the text-channel
                } else {
                    if perm.allow.read_messages() {
                        members_in_vc.remove(i);
                    }
                }
            }
            _ => continue,
        }
    }

    // members_in_vc at this point is considered as in the voice channel,
    // but they don't have access to the text channel
    for member in members_in_vc.iter() {
        grant_access(ctx, &text, member.user.id).await;
    }
}

fn in_vc(user: UserId, members: &Vec<Member>) -> (bool, usize) {
    let mut i: usize = 0;
    for member in members.iter() {
        if member.user.id == user {
            return (true, i);
        }
        i += 1;
    }
    return (false, 0);
}

fn get_room(serving: &Serving, id: &ChannelId) -> Option<Room> {
    for room in serving.rooms.iter() {
        if room.voice_id.as_u64() == id.as_u64() {
            return Some(room.clone());
        }
    }
    return None;
}
