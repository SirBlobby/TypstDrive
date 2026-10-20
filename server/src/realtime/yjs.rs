use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use futures_util::stream::{Stream, StreamExt};
use futures_util::SinkExt;
use tokio::sync::{Mutex, RwLock};
use yrs::sync::protocol::{Message, MessageReader, SyncMessage};
use yrs::sync::Awareness;
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::{Doc, ReadTxn, Transact, Update};
use yrs_axum::broadcast::BroadcastGroup;
use yrs_axum::ws::AxumSink;

use crate::realtime::relay;
use crate::sharing::access::{self, AccessRole, Resource};
use crate::desktop::devices::{notify_devices, DeviceEvent};
use crate::AppState;
use crate::db::sql;

pub struct ViewerFilterStream {
    inner: futures_util::stream::SplitStream<axum::extract::ws::WebSocket>,
    read_only: bool,
    revoked: Arc<AtomicBool>,
    state: AppState,
    room_id: String,
    user_id: Option<String>,
}

fn contains_document_change(bytes: &[u8]) -> bool {
    let mut decoder = DecoderV1::from(bytes);
    MessageReader::new(&mut decoder).any(|message| {
        matches!(message, Ok(Message::Sync(SyncMessage::Update(_))))
    })
}

fn is_read_only_frame(bytes: &[u8]) -> bool {
    let mut decoder = DecoderV1::from(bytes);
    let mut saw_message = false;
    for message in MessageReader::new(&mut decoder) {
        match message {
            Ok(Message::Sync(SyncMessage::SyncStep1(_))) | Ok(Message::Awareness(_)) | Ok(Message::AwarenessQuery) => {
                saw_message = true;
            }
            _ => return false,
        }
    }
    saw_message
}

impl Stream for ViewerFilterStream {
    type Item = Result<Vec<u8>, yrs::sync::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if self.revoked.load(Ordering::SeqCst) {
                return std::task::Poll::Ready(None);
            }
            match futures_util::ready!(std::pin::Pin::new(&mut self.inner).poll_next(cx)) {
                Some(Ok(axum::extract::ws::Message::Binary(bytes))) => {
                    if self.read_only && !is_read_only_frame(&bytes) {
                        continue;
                    }
                    if let Some(user_id) = self.user_id.as_deref().filter(|_| contains_document_change(&bytes)) {
                        crate::drive::activity::note_live_edit(&self.room_id, user_id);
                    }
                    relay::forward_client_frame(&self.state, &self.room_id, &bytes);
                    return std::task::Poll::Ready(Some(Ok(bytes.to_vec())));
                }
                Some(Ok(axum::extract::ws::Message::Close(_))) | None => return std::task::Poll::Ready(None),
                Some(Ok(_)) => continue,
                Some(Err(e)) => return std::task::Poll::Ready(Some(Err(yrs::sync::Error::Other(Box::new(e))))),
            }
        }
    }
}

struct YjsSaveTarget {
    room_id: String,
    table: &'static str,
    row_id: String,
    project_id: Option<String>,
    owner_id: String,
    event: DeviceEvent,
}

struct YjsRoom {
    resource: Resource,
    role: AccessRole,
    initial_content: Option<Vec<u8>>,
    save_target: YjsSaveTarget,
}

async fn resolve_room(state: &AppState, room_id: &str, user_id: &Option<String>) -> Option<YjsRoom> {
    if let Some(rest) = room_id.strip_prefix("project:") {
        let (project_id, file_id) = rest.split_once(':')?;
        let (project, role) = access::project_access(state, project_id, user_id).await?;
        let (content,) = sqlx::query_as::<_, (Option<Vec<u8>>,)>(
            sql("SELECT content FROM project_files WHERE id = $1 AND project_id = $2")
        )
        .bind(file_id)
        .bind(project_id)
        .fetch_optional(&state.db)
        .await
        .ok()??;

        return Some(YjsRoom {
            resource: Resource::Project(project_id.to_string()),
            role,
            initial_content: content,
            save_target: YjsSaveTarget {
                room_id: room_id.to_string(),
                table: "project_files",
                row_id: file_id.to_string(),
                project_id: Some(project_id.to_string()),
                owner_id: project.owner_id,
                event: DeviceEvent::project(project_id),
            },
        });
    }

    let (document, role) = access::document_role(state, room_id, user_id).await?;
    Some(YjsRoom {
        resource: Resource::Document(room_id.to_string()),
        role,
        initial_content: document.content,
        save_target: YjsSaveTarget {
            room_id: room_id.to_string(),
            table: "documents",
            row_id: room_id.to_string(),
            project_id: None,
            owner_id: document.owner_id,
            event: DeviceEvent::document(room_id),
        },
    })
}

fn spawn_autosave(state: AppState, awareness: Arc<RwLock<Awareness>>, target: YjsSaveTarget) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        let mut last_content: Option<Vec<u8>> = None;
        loop {
            interval.tick().await;
            let doc = awareness.read().await;
            let content = doc.doc().transact().encode_state_as_update_v1(&yrs::StateVector::default());
            drop(doc);

            if last_content.as_ref() == Some(&content) {
                continue;
            }

            let query = if target.table == "project_files" {
                "UPDATE project_files SET content = $1, updated_at = $2 WHERE id = $3"
            } else {
                "UPDATE documents SET content = $1, updated_at = $2 WHERE id = $3"
            };
            let result = sqlx::query(sql(query))
                .bind(&content)
                .bind(crate::db::timestamp_now())
                .bind(&target.row_id)
                .execute(&state.db)
                .await;

            if result.is_ok() {
                last_content = Some(content);
                let editor = crate::drive::activity::live_editor(&target.room_id);
                match &target.project_id {
                    Some(project_id) => crate::drive::activity::mark_project_edited(&state, project_id, editor.as_deref()).await,
                    None => crate::drive::activity::mark_document_edited(&state, &target.row_id, editor.as_deref()).await,
                }
                notify_devices(&state, &target.owner_id, target.event.clone()).await;
            }
        }
    });
}

async fn room_broadcast(state: &AppState, room_id: &str, room: YjsRoom) -> (Arc<BroadcastGroup>, Resource, AccessRole) {
    let YjsRoom { resource, role, initial_content, save_target } = room;
    let mut bcast_map = state.bcast_map.lock().await;
    if let Some(existing) = bcast_map.get(room_id) {
        return (existing.clone(), resource, role);
    }

    let ydoc = Doc::new();
    if let Some(content) = initial_content {
        if let Ok(update) = Update::decode_v1(&content) {
            ydoc.transact_mut().apply_update(update);
        }
    }

    let awareness = Arc::new(RwLock::new(Awareness::new(ydoc)));
    let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 10).await);
    bcast_map.insert(room_id.to_string(), bcast.clone());
    crate::realtime::live_documents::register(state, room_id, awareness.clone()).await;
    relay::request_missing_updates(state, room_id, &awareness).await;
    spawn_autosave(state.clone(), awareness, save_target);

    (bcast, resource, role)
}

pub async fn yjs_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
) -> Response {
    let (user_id, device_token_id) = match jar.get("session_user_id").map(|c| c.value().to_string()) {
        Some(uid) => (Some(uid), None),
        None => match params.get("token") {
            Some(token) => match crate::desktop::device_token_for(&state, token).await {
                Some((token_id, uid)) => (Some(uid), Some(token_id)),
                None => (None, None),
            },
            None => (None, None),
        },
    };

    let room = match resolve_room(&state, &room_id, &user_id).await {
        Some(room) => room,
        None => return access::forbidden().into_response(),
    };

    let (bcast, resource, role) = room_broadcast(&state, &room_id, room).await;

    ws.on_upgrade(move |socket| async move {
        let session = access::register_session(&state, resource, user_id.clone(), device_token_id, role.can_edit()).await;

        let (sink, stream) = socket.split();
        let sink = Arc::new(Mutex::new(AxumSink(sink)));
        let revoked_flag = Arc::new(AtomicBool::new(false));

        let filtered_stream = ViewerFilterStream {
            inner: stream,
            read_only: !role.can_edit(),
            revoked: revoked_flag.clone(),
            state: state.clone(),
            room_id: room_id.clone(),
            user_id: user_id.clone(),
        };

        let subscription = bcast.subscribe(sink.clone(), filtered_stream);
        tokio::select! {
            result = subscription.completed() => {
                if let Err(e) = result {
                    tracing::debug!("collaboration socket for {room_id} closed: {e}");
                }
            }
            _ = session.revoked => {
                revoked_flag.store(true, Ordering::SeqCst);
                let _ = sink.lock().await.close().await;
                tracing::info!("revoked collaboration socket for {room_id}");
            }
        }

        access::unregister_session(&state, &session.resource, session.session_id).await;
    })
}
