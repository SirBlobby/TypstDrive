use std::sync::Arc;

use tokio::sync::RwLock;
use yrs::sync::protocol::{Message, MessageReader, SyncMessage};
use yrs::sync::Awareness;
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, Transact, Update};

use crate::realtime::live_documents;
use crate::AppState;

fn decode_messages(frame: &[u8]) -> Vec<Message> {
    let mut decoder = DecoderV1::from(frame);
    MessageReader::new(&mut decoder).flatten().collect()
}

pub fn forward_client_frame(state: &AppState, room_id: &str, frame: &[u8]) {
    if !state.cluster.is_enabled() {
        return;
    }
    let mut relayed = Vec::new();
    for message in decode_messages(frame) {
        let shared = match message {
            Message::Sync(SyncMessage::Update(update)) | Message::Sync(SyncMessage::SyncStep2(update)) => {
                Message::Sync(SyncMessage::Update(update))
            }
            Message::Awareness(update) => Message::Awareness(update),
            _ => continue,
        };
        relayed.extend(shared.encode_v1());
    }
    if !relayed.is_empty() {
        state.cluster.publish_room(room_id, &relayed);
    }
}

pub fn publish_update(state: &AppState, room_id: &str, update: Vec<u8>) {
    if state.cluster.is_enabled() && !update.is_empty() {
        state.cluster.publish_room(room_id, &Message::Sync(SyncMessage::Update(update)).encode_v1());
    }
}

pub async fn request_missing_updates(state: &AppState, room_id: &str, awareness: &Arc<RwLock<Awareness>>) {
    if !state.cluster.is_enabled() {
        return;
    }
    let state_vector = awareness.read().await.doc().transact().state_vector();
    state.cluster.publish_room(room_id, &Message::Sync(SyncMessage::SyncStep1(state_vector)).encode_v1());
}

pub async fn apply_remote_frame(state: &AppState, room_id: &str, frame: &[u8]) {
    let Some(awareness) = live_documents::room(state, room_id).await else {
        return;
    };

    for message in decode_messages(frame) {
        match message {
            Message::Sync(SyncMessage::Update(update)) | Message::Sync(SyncMessage::SyncStep2(update)) => {
                let awareness = awareness.write().await;
                if let Ok(update) = Update::decode_v1(&update) {
                    awareness.doc().transact_mut().apply_update(update);
                }
            }
            Message::Sync(SyncMessage::SyncStep1(remote_state)) => {
                let missing = awareness.write().await.doc().transact().encode_diff_v1(&remote_state);
                publish_update(state, room_id, missing);
            }
            Message::Awareness(update) => {
                let mut awareness = awareness.write().await;
                let _ = awareness.apply_update(update);
            }
            _ => {}
        }
    }
}
