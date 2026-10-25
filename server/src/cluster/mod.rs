mod listener;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::desktop::devices::DeviceEvent;
use crate::AppState;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClusterEvent {
    RevalidateDocument { document_id: String },
    RevalidateProject { project_id: String },
    RevokeUser { user_id: String },
    RevokeDevice { device_token_id: String },
    DeviceNotification { user_id: String, event: DeviceEvent },
    ReloadThemes { owner_id: Option<String> },
}

type OutgoingMessage = (String, Vec<u8>);

pub struct Cluster {
    instance_id: String,
    channel_prefix: String,
    client: Option<redis::Client>,
    outgoing: Option<mpsc::UnboundedSender<OutgoingMessage>>,
    connected: Arc<AtomicBool>,
}

impl Cluster {
    pub fn from_environment() -> Self {
        let instance_id = Uuid::new_v4().to_string();
        let channel_prefix = std::env::var("REDIS_CHANNEL_PREFIX")
            .ok()
            .filter(|prefix| !prefix.trim().is_empty())
            .unwrap_or_else(|| "typstdrive".to_string());
        let connected = Arc::new(AtomicBool::new(false));

        let client = std::env::var("REDIS_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
            .map(|url| redis::Client::open(url.trim()).expect("Invalid REDIS_URL"));

        let outgoing = client.clone().map(|client| {
            let (sender, receiver) = mpsc::unbounded_channel();
            tokio::spawn(publish_loop(client, receiver));
            sender
        });
        if outgoing.is_some() {
            tracing::info!("Real-time coordination through Redis is enabled for instance {instance_id}");
        }

        Self { instance_id, channel_prefix, client, outgoing, connected }
    }

    pub fn is_enabled(&self) -> bool {
        self.outgoing.is_some()
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn room_channel_prefix(&self) -> String {
        format!("{}:room:", self.channel_prefix)
    }

    fn events_channel(&self) -> String {
        format!("{}:events", self.channel_prefix)
    }

    fn publish(&self, channel: String, body: &[u8]) {
        let Some(outgoing) = &self.outgoing else {
            return;
        };
        let mut payload = Vec::with_capacity(self.instance_id.len() + body.len());
        payload.extend_from_slice(self.instance_id.as_bytes());
        payload.extend_from_slice(body);
        let _ = outgoing.send((channel, payload));
    }

    pub fn publish_room(&self, room_id: &str, frame: &[u8]) {
        if self.is_enabled() {
            self.publish(format!("{}{room_id}", self.room_channel_prefix()), frame);
        }
    }

    pub fn publish_event(&self, event: &ClusterEvent) {
        if !self.is_enabled() {
            return;
        }
        if let Ok(body) = serde_json::to_vec(event) {
            self.publish(self.events_channel(), &body);
        }
    }

    fn foreign_body<'a>(&self, payload: &'a [u8]) -> Option<&'a [u8]> {
        let (sender, body) = payload.split_at_checked(self.instance_id.len())?;
        (sender != self.instance_id.as_bytes()).then_some(body)
    }
}

async fn publish_loop(client: redis::Client, mut receiver: mpsc::UnboundedReceiver<OutgoingMessage>) {
    let mut connection: Option<redis::aio::MultiplexedConnection> = None;
    while let Some((channel, payload)) = receiver.recv().await {
        if connection.is_none() {
            connection = match client.get_multiplexed_async_connection().await {
                Ok(new_connection) => Some(new_connection),
                Err(error) => {
                    tracing::warn!("Could not connect to Redis to publish: {error}");
                    None
                }
            };
        }
        let Some(active) = connection.as_mut() else {
            continue;
        };
        let published: redis::RedisResult<()> = active.publish(&channel, payload).await;
        if let Err(error) = published {
            tracing::warn!("Redis publish failed: {error}");
            connection = None;
        }
    }
}

pub fn start(state: AppState) {
    let Some(client) = state.cluster.client.clone() else {
        return;
    };
    let connected = state.cluster.connected.clone();
    tokio::spawn(listener::run(client, state, connected));
}
