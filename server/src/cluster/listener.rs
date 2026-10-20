use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;

use super::ClusterEvent;
use crate::desktop::devices;
use crate::realtime::relay;
use crate::sharing::access::{self, Resource};
use crate::AppState;

const RECONNECT_DELAY: Duration = Duration::from_secs(3);

pub async fn run(client: redis::Client, state: AppState, connected: Arc<AtomicBool>) {
    let pattern = format!("{}:*", state.cluster.channel_prefix);
    loop {
        match client.get_async_pubsub().await {
            Ok(mut pubsub) => match pubsub.psubscribe(&pattern).await {
                Ok(()) => {
                    connected.store(true, Ordering::Relaxed);
                    tracing::info!("Subscribed to Redis channels matching {pattern}");
                    let mut messages = pubsub.on_message();
                    while let Some(message) = messages.next().await {
                        let channel = message.get_channel_name().to_string();
                        dispatch(&state, &channel, message.get_payload_bytes()).await;
                    }
                }
                Err(error) => tracing::warn!("Redis subscribe failed: {error}"),
            },
            Err(error) => tracing::warn!("Could not connect to Redis: {error}"),
        }
        connected.store(false, Ordering::Relaxed);
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

async fn dispatch(state: &AppState, channel: &str, payload: &[u8]) {
    let Some(body) = state.cluster.foreign_body(payload) else {
        return;
    };

    if let Some(room_id) = channel.strip_prefix(&state.cluster.room_channel_prefix()) {
        relay::apply_remote_frame(state, room_id, body).await;
    } else if channel == state.cluster.events_channel() {
        if let Ok(event) = serde_json::from_slice::<ClusterEvent>(body) {
            apply_event(state, event).await;
        }
    }
}

async fn apply_event(state: &AppState, event: ClusterEvent) {
    match event {
        ClusterEvent::RevalidateDocument { document_id } => {
            access::revalidate_local(state, Resource::Document(document_id)).await
        }
        ClusterEvent::RevalidateProject { project_id } => {
            access::revalidate_local(state, Resource::Project(project_id)).await
        }
        ClusterEvent::RevokeUser { user_id } => access::revoke_user_sessions_local(state, &user_id).await,
        ClusterEvent::RevokeDevice { device_token_id } => {
            access::revoke_device_sessions_local(state, &device_token_id).await
        }
        ClusterEvent::DeviceNotification { user_id, event } => devices::deliver_local(state, &user_id, event).await,
    }
}
