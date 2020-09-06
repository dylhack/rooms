use crate::bot::util::{get_channels, grant_access, revoke_access};
use crate::config::{Room, Serving};
use log::{info};
use serenity::client::Context;
use serenity::model::prelude::*;

// review_state reviews a member's voice state and checks if the voice channel they joined or left
// needs synced by sync_rooms.
pub async fn review_state(ctx: &Context, serving: &Serving, state: &VoiceState) {
    if let Some(channel_id) = state.channel_id {
        if let Some(room) = get_room(&serving, &channel_id) {
            sync_room(&ctx, &room).await;
        }
    }
}

// sync_room is where all the magic happens. It will make sure the people in the voice channel can
// see the linked text-channel. It also revokes access to the text-channel for the ones that aren't
// in the voice-channel.
async fn sync_room(ctx: &Context, room: &Room) {
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

    info!("Syncing {} and #{}", voice.name, text.name);
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

// Check if a given user ID is a voice channel.
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

// Get the text-channel associated with a voice channel in a guild (Serving).
fn get_room(serving: &Serving, id: &ChannelId) -> Option<Room> {
    for room in serving.rooms.iter() {
        if room.voice_id.as_u64() == id.as_u64() {
            return Some(room.clone());
        }
    }
    return None;
}
