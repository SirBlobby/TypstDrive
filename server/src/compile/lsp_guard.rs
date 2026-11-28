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
const PIN_MAIN_ID: &str = "typstdrive:pin-main";

pub struct LspGuard {
    root_uri: String,
    root_path: String,
    package_path: String,
    entry_path: Option<String>,
    pending_configuration_ids: Mutex<HashSet<String>>,
}

fn message_id(value: &Value) -> Option<String> {
    value.get("id").map(Value::to_string)
}

fn is_uri_key(key: &str) -> bool {
    key.to_ascii_lowercase().ends_with("uri")
}

impl LspGuard {
    pub fn new(root: &Path, package_path: &Path, entry: Option<&str>) -> Self {
        let root_path = root.display().to_string();
        Self {
            root_uri: format!("file://{root_path}"),
            root_path,
            package_path: package_path.display().to_string(),
            entry_path: entry.map(|entry| root.join(entry).display().to_string()),
            pending_configuration_ids: Mutex::new(HashSet::new()),
        }
    }

    pub fn root_uri(&self) -> &str {
        &self.root_uri
    }

    pub fn to_server(&self, message: &str) -> Vec<String> {
        let Ok(mut value) = serde_json::from_str::<Value>(message) else {
            return Vec::new();
        };
        let method = value.get("method").and_then(Value::as_str).map(str::to_owned);
        match method.as_deref() {
            Some(method) if BLOCKED_METHODS.contains(&method) => return Vec::new(),
            Some("initialize") => self.lock_initialize(&mut value),
            Some(_) => {}
            None => self.clear_configuration_response(&mut value),
        }
        if !self.uris_inside_root(&value) {
            return Vec::new();
        }
        let mut messages = vec![value.to_string()];
        if let (Some("initialized"), Some(entry)) = (method.as_deref(), &self.entry_path) {
            let pin = json!({ "jsonrpc": "2.0", "id": PIN_MAIN_ID, "method": "workspace/executeCommand", "params": { "command": "tinymist.pinMain", "arguments": [entry] } });
            messages.push(pin.to_string());
        }
        messages
    }

    pub fn to_client(&self, message: &str) -> bool {
        let Ok(value) = serde_json::from_str::<Value>(message) else {
            return true;
        };
        if value.get("id").and_then(Value::as_str) == Some(PIN_MAIN_ID) && value.get("method").is_none() {
            return false;
        }
        if value.get("method").and_then(Value::as_str) == Some(CONFIGURATION_REQUEST) {
            if let (Some(id), Ok(mut pending)) = (message_id(&value), self.pending_configuration_ids.lock()) {
                pending.insert(id);
            }
        }
        true
    }

    fn lock_initialize(&self, value: &mut Value) {
        let Some(params) = value.get_mut("params").and_then(Value::as_object_mut) else {
            return;
        };
        params.insert("rootUri".to_string(), json!(self.root_uri));
        params.insert("rootPath".to_string(), json!(self.root_path));
        params.insert("workspaceFolders".to_string(), json!([{ "uri": self.root_uri, "name": "workspace" }]));
        params.insert(
            "initializationOptions".to_string(),
            json!({ "rootPath": self.root_path, "exportPdf": "never", "typstExtraArgs": ["--package-path", self.package_path] }),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(entry: Option<&str>) -> LspGuard {
        LspGuard::new(Path::new("/ws/project"), Path::new("/ws/packages"), entry)
    }

    fn parse(message: &str) -> Value {
        serde_json::from_str(message).unwrap()
    }

    #[test]
    fn initialize_is_locked_to_the_workspace() {
        let message = json!({ "id": 1, "method": "initialize", "params": { "rootUri": "file:///etc", "initializationOptions": { "typstExtraArgs": ["--root", "/"] } } });
        let forwarded = guard(None).to_server(&message.to_string());
        let params = &parse(&forwarded[0])["params"];
        assert_eq!(params["rootUri"], "file:///ws/project");
        assert_eq!(params["initializationOptions"]["typstExtraArgs"], json!(["--package-path", "/ws/packages"]));
    }

    #[test]
    fn initialized_pins_the_entrypoint() {
        let message = json!({ "method": "initialized", "params": {} }).to_string();
        assert_eq!(guard(None).to_server(&message).len(), 1);

        let forwarded = guard(Some("main.typ")).to_server(&message);
        assert_eq!(forwarded.len(), 2);
        let pin = parse(&forwarded[1]);
        assert_eq!(pin["params"]["command"], "tinymist.pinMain");
        assert_eq!(pin["params"]["arguments"], json!(["/ws/project/main.typ"]));
    }

    #[test]
    fn pin_response_is_hidden_from_the_client() {
        let guard = guard(Some("main.typ"));
        assert!(!guard.to_client(&json!({ "id": PIN_MAIN_ID, "result": null }).to_string()));
        assert!(guard.to_client(&json!({ "id": 1, "result": null }).to_string()));
    }

    #[test]
    fn blocked_methods_and_outside_uris_are_dropped() {
        let guard = guard(None);
        let command = json!({ "id": 2, "method": "workspace/executeCommand", "params": { "command": "tinymist.pinMain" } });
        assert!(guard.to_server(&command.to_string()).is_empty());
        for uri in ["file:///etc/passwd", "file:///ws/project/../x.typ", "file:///ws/project/%2e%2e/x.typ", "file:///ws/projectx/a.typ"] {
            let open = json!({ "method": "textDocument/didOpen", "params": { "textDocument": { "uri": uri } } });
            assert!(guard.to_server(&open.to_string()).is_empty(), "{uri}");
        }
        let inside = json!({ "method": "textDocument/didOpen", "params": { "textDocument": { "uri": "file:///ws/project/a/b.bib" } } });
        assert_eq!(guard.to_server(&inside.to_string()).len(), 1);
    }

    #[test]
    fn configuration_responses_are_nulled() {
        let guard = guard(None);
        guard.to_client(&json!({ "id": 7, "method": CONFIGURATION_REQUEST, "params": { "items": [{}, {}] } }).to_string());
        let reply = guard.to_server(&json!({ "id": 7, "result": [{ "rootPath": "/" }, 1] }).to_string());
        assert_eq!(parse(&reply[0])["result"], json!([null, null]));
    }
}
