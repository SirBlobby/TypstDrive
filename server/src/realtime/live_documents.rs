use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact, Update};

use crate::realtime::relay;
use crate::AppState;

const TEXT_NAME: &str = "typst";

pub type LiveDocuments = HashMap<String, Arc<RwLock<Awareness>>>;

pub fn document_room(document_id: &str) -> String {
    document_id.to_string()
}

pub fn project_file_room(project_id: &str, file_id: &str) -> String {
    format!("project:{project_id}:{file_id}")
}

pub async fn register(state: &AppState, room_id: &str, awareness: Arc<RwLock<Awareness>>) {
    state.live_documents.lock().await.insert(room_id.to_string(), awareness);
}

pub async fn room(state: &AppState, room_id: &str) -> Option<Arc<RwLock<Awareness>>> {
    state.live_documents.lock().await.get(room_id).cloned()
}

pub async fn live_text(state: &AppState, room_id: &str) -> Option<String> {
    let awareness = room(state, room_id).await?;
    let awareness = awareness.read().await;
    let doc = awareness.doc();
    let text = doc.get_or_insert_text(TEXT_NAME);
    let content = text.get_string(&doc.transact());
    Some(content)
}

fn changed_range(current: &str, replacement: &str) -> (usize, usize, usize) {
    let mut prefix = current
        .bytes()
        .zip(replacement.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    while !current.is_char_boundary(prefix) || !replacement.is_char_boundary(prefix) {
        prefix -= 1;
    }

    let maximum_suffix = (current.len() - prefix).min(replacement.len() - prefix);
    let mut suffix = current
        .bytes()
        .rev()
        .zip(replacement.bytes().rev())
        .take(maximum_suffix)
        .take_while(|(left, right)| left == right)
        .count();
    while !current.is_char_boundary(current.len() - suffix) || !replacement.is_char_boundary(replacement.len() - suffix) {
        suffix -= 1;
    }

    (prefix, current.len() - prefix - suffix, replacement.len() - suffix)
}

fn apply_replacement(doc: &Doc, replacement: &str) -> bool {
    let text = doc.get_or_insert_text(TEXT_NAME);
    let mut transaction = doc.transact_mut();
    let current = text.get_string(&transaction);
    if current == replacement {
        return false;
    }

    let (start, removed_length, replacement_end) = changed_range(&current, replacement);
    if removed_length > 0 {
        text.remove_range(&mut transaction, start as u32, removed_length as u32);
    }
    if replacement_end > start {
        text.insert(&mut transaction, start as u32, &replacement[start..replacement_end]);
    }
    true
}

pub fn merge_text_into_blob(stored: Option<&[u8]>, replacement: &str) -> Vec<u8> {
    let doc = Doc::new();
    if let Some(update) = stored.and_then(|bytes| Update::decode_v1(bytes).ok()) {
        doc.transact_mut().apply_update(update);
    }
    apply_replacement(&doc, replacement);
    let encoded = doc.transact().encode_state_as_update_v1(&StateVector::default());
    encoded
}

pub async fn replace_text(state: &AppState, room_id: &str, replacement: &str) -> bool {
    let Some(awareness) = room(state, room_id).await else {
        return false;
    };
    let update = {
        let awareness = awareness.read().await;
        let doc = awareness.doc();
        let state_before = doc.transact().state_vector();
        if !apply_replacement(doc, replacement) {
            return true;
        }
        let difference = doc.transact().encode_diff_v1(&state_before);
        difference
    };
    relay::publish_update(state, room_id, update);
    true
}

pub async fn store_text(state: &AppState, room_id: &str, stored: Option<&[u8]>, replacement: &str) -> Vec<u8> {
    if replace_text(state, room_id, replacement).await {
        if let Some(awareness) = room(state, room_id).await {
            let awareness = awareness.read().await;
            let encoded = awareness.doc().transact().encode_state_as_update_v1(&StateVector::default());
            return encoded;
        }
    }
    merge_text_into_blob(stored, replacement)
}
