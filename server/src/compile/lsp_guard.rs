use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use serde_json::{json, Value};

const BLOCKED_METHODS: &[&str] = &[
    "workspace/executeCommand",
    "workspace/didChangeConfiguration",
    "workspace/didChangeWorkspaceFolders",
    "workspace/didChangeWatchedFiles",
    "workspace/didCreateFiles",
    "workspace/didRenameFiles",
    "workspace/didDeleteFiles",
    "workspace/willCreateFiles",
    "workspace/willRenameFiles",
    "workspace/willDeleteFiles",
];

const CONFIGURATION_REQUEST: &str = "workspace/configuration";

pub struct LspGuard {
    root_uri: String,
    root_path: String,
    pending_configuration_ids: Mutex<HashSet<String>>,
}

fn message_id(value: &Value) -> Option<String> {
    value.get("id").map(Value::to_string)
}

fn is_uri_key(key: &str) -> bool {
    key.to_ascii_lowercase().ends_with("uri")
}

impl LspGuard {
    pub fn new(root: &Path) -> Self {
        let root_path = root.display().to_string();
        Self {
            root_uri: format!("file://{root_path}"),
            root_path,
            pending_configuration_ids: Mutex::new(HashSet::new()),
        }
    }

    pub fn root_uri(&self) -> &str {
        &self.root_uri
    }

    pub fn from_client(&self, message: &str) -> Option<String> {
        let mut value: Value = serde_json::from_str(message).ok()?;
        match value.get("method").and_then(Value::as_str) {
            Some(method) if BLOCKED_METHODS.contains(&method) => return None,
            Some("initialize") => self.lock_initialize(&mut value),
            Some(_) => {}
            None => self.clear_configuration_response(&mut value),
        }
        if !self.uris_inside_root(&value) {
            return None;
        }
        Some(value.to_string())
    }

    pub fn note_server_message(&self, message: &str) {
        let Ok(value) = serde_json::from_str::<Value>(message) else {
            return;
        };
        if value.get("method").and_then(Value::as_str) != Some(CONFIGURATION_REQUEST) {
            return;
        }
        if let (Some(id), Ok(mut pending)) = (message_id(&value), self.pending_configuration_ids.lock()) {
            pending.insert(id);
        }
    }

    fn lock_initialize(&self, value: &mut Value) {
        let Some(params) = value.get_mut("params").and_then(Value::as_object_mut) else {
            return;
        };
        params.insert("rootUri".to_string(), json!(self.root_uri));
        params.insert("rootPath".to_string(), json!(self.root_path));
        params.insert("workspaceFolders".to_string(), json!([{ "uri": self.root_uri, "name": "workspace" }]));
        params.insert("initializationOptions".to_string(), json!({ "rootPath": self.root_path, "exportPdf": "never" }));
    }

    fn clear_configuration_response(&self, value: &mut Value) {
        let Some(id) = message_id(value) else {
            return;
        };
        let was_configuration = self
            .pending_configuration_ids
            .lock()
            .map(|mut pending| pending.remove(&id))
            .unwrap_or(false);
        if !was_configuration {
            return;
        }
        if let Some(result) = value.get_mut("result") {
            *result = match result.as_array() {
                Some(items) => Value::Array(vec![Value::Null; items.len()]),
                None => Value::Null,
            };
        }
    }

    fn is_allowed_uri(&self, uri: &str) -> bool {
        let lower = uri.to_ascii_lowercase();
        let inside_root = uri == self.root_uri || uri.starts_with(&format!("{}/", self.root_uri));
        inside_root && !uri.contains("..") && !lower.contains("%2e") && !lower.contains("%2f") && !lower.contains("%5c")
    }

    fn uris_inside_root(&self, value: &Value) -> bool {
        match value {
            Value::Object(fields) => fields.iter().all(|(key, field)| match field {
                Value::String(uri) if is_uri_key(key) => self.is_allowed_uri(uri),
                other => self.uris_inside_root(other),
            }),
            Value::Array(items) => items.iter().all(|item| self.uris_inside_root(item)),
            _ => true,
        }
    }
}
