use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs_axum::broadcast::BroadcastGroup;

use crate::AppState;

pub struct LiveRoom {
    pub broadcast: Arc<BroadcastGroup>,
    connections: usize,
}

pub type LiveRooms = HashMap<String, LiveRoom>;

pub async fn join_existing(state: &AppState, room_id: &str) -> Option<Arc<BroadcastGroup>> {
    let mut rooms = state.live_rooms.lock().await;
    let room = rooms.get_mut(room_id)?;
    room.connections += 1;
    Some(room.broadcast.clone())
}

pub async fn join_or_create(
    state: &AppState,
    room_id: &str,
    awareness: Arc<RwLock<Awareness>>,
) -> (Arc<BroadcastGroup>, bool) {
    let mut rooms = state.live_rooms.lock().await;
    if let Some(room) = rooms.get_mut(room_id) {
        room.connections += 1;
        return (room.broadcast.clone(), false);
    }
    let broadcast = Arc::new(BroadcastGroup::new(awareness.clone(), 10).await);
    rooms.insert(room_id.to_string(), LiveRoom { broadcast: broadcast.clone(), connections: 1 });
    crate::realtime::live_documents::register(state, room_id, awareness).await;
    (broadcast, true)
}

pub async fn leave(state: &AppState, room_id: &str) {
    if let Some(room) = state.live_rooms.lock().await.get_mut(room_id) {
        room.connections = room.connections.saturating_sub(1);
    }
}

pub async fn close_if_idle(state: &AppState, room_id: &str) -> bool {
    let mut rooms = state.live_rooms.lock().await;
    match rooms.get(room_id) {
        Some(room) if room.connections > 0 => false,
        _ => {
            rooms.remove(room_id);
            crate::realtime::live_documents::unregister(state, room_id).await;
            true
        }
    }
}
