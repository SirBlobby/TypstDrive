use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{HeaderMap, StatusCode},
    Json,
};
use axum_extra::extract::cookie::SignedCookieJar;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use uuid::Uuid;

use crate::AppState;

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .map(|value| value[7..].to_string())
}

fn get_user_id(jar: &SignedCookieJar) -> Option<String> {
    jar.get("session_user_id").map(|c| c.value().to_string())
}

async fn authenticate_device(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, String), (StatusCode, String)> {
    let token = bearer_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing Authorization header".to_string(),
    ))?;

    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT id, user_id FROM device_tokens WHERE token_hash = ?",
    )
    .bind(hash_token(&token))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (device_id, user_id) = row.ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid device token".to_string(),
    ))?;

    Ok((user_id, device_id))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceEvent {
    pub kind: String,
    pub project_id: Option<String>,
    pub document_id: Option<String>,
}

impl DeviceEvent {
    pub fn project(project_id: &str) -> Self {
        Self {
            kind: "project".to_string(),
            project_id: Some(project_id.to_string()),
            document_id: None,
        }
    }

    pub fn document(document_id: &str) -> Self {
        Self {
            kind: "document".to_string(),
            project_id: None,
            document_id: Some(document_id.to_string()),
        }
    }

    pub fn structure() -> Self {
        Self {
            kind: "structure".to_string(),
            project_id: None,
            document_id: None,
        }
    }
}

pub async fn notify_devices(state: &AppState, user_id: &str, event: DeviceEvent) {
    let events = state.device_events.lock().await;
    if let Some(sender) = events.get(user_id) {
        let _ = sender.send(event);
    }
}

pub struct DevicePresence {
    pub connected_since: String,
    pub connection_id: String,
    pub stop: watch::Sender<bool>,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let (user_id, device_id) = authenticate_device(&state, &headers).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(state, socket, user_id, device_id)))
}

async fn handle_socket(state: AppState, socket: WebSocket, user_id: String, device_id: String) {
    let mut receiver = {
        let mut events = state.device_events.lock().await;
        let sender = events
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(16).0)
            .clone();
        sender.subscribe()
    };

    let connection_id = Uuid::new_v4().to_string();
    let (stop_tx, mut stop_rx) = watch::channel(false);

    {
        let mut presence = state.device_presence.lock().await;
        presence.insert(
            device_id.clone(),
            DevicePresence {
                connected_since: chrono::Utc::now().to_rfc3339(),
                connection_id: connection_id.clone(),
                stop: stop_tx,
            },
        );
    }

    let (mut sink, mut stream) = socket.split();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(20));
    heartbeat.tick().await;

    loop {
        tokio::select! {
            event = receiver.recv() => {
                match event {
                    Ok(event) => {
                        let Ok(payload) = serde_json::to_string(&event) else { continue };
                        if sink.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = heartbeat.tick() => {
                if sink.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            _ = stop_rx.changed() => {
                break;
            }
        }
    }

    let mut presence = state.device_presence.lock().await;
    if presence
        .get(&device_id)
        .map(|entry| entry.connection_id == connection_id)
        .unwrap_or(false)
    {
        presence.remove(&device_id);
    }
}

#[derive(Serialize)]
pub struct DeviceView {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub connected: bool,
    pub connected_since: Option<String>,
}

pub async fn list_devices(
    State(state): State<AppState>,
    jar: SignedCookieJar,
) -> Result<Json<Vec<DeviceView>>, (StatusCode, String)> {
    let user_id =
        get_user_id(&jar).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, name, created_at, last_used_at FROM device_tokens WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let presence = state.device_presence.lock().await;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, created_at, last_used_at)| {
                let entry = presence.get(&id);
                DeviceView {
                    connected: entry.is_some(),
                    connected_since: entry.map(|e| e.connected_since.clone()),
                    id,
                    name,
                    created_at,
                    last_used_at,
                }
            })
            .collect(),
    ))
}

pub async fn revoke_device(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id =
        get_user_id(&jar).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let result = sqlx::query("DELETE FROM device_tokens WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Device not found".to_string()));
    }

    let presence = state.device_presence.lock().await;
    if let Some(entry) = presence.get(&id) {
        let _ = entry.stop.send(true);
    }

    Ok(StatusCode::NO_CONTENT)
}
