use axum_extra::extract::cookie::Key;

use crate::sealed_secrets;

const SEALED_NAME: &str = "zotero_key";

pub fn seal(key: &Key, value: &str) -> String {
    sealed_secrets::seal(key, SEALED_NAME, value)
}

pub fn open(key: &Key, sealed: &str) -> Option<String> {
    sealed_secrets::open(key, SEALED_NAME, sealed)
}
