use std::sync::LazyLock;

use argon2::password_hash::phc::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::http::StatusCode;

pub const NO_PASSWORD: &str = "!";
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 1024;

static UNKNOWN_USER_HASH: LazyLock<String> = LazyLock::new(|| hash_blocking("unknown-user-placeholder").unwrap_or_default());

fn hash_blocking(password: &str) -> Result<String, String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_blocking(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed_hash) => Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok(),
        Err(_) => false,
    }
}

pub fn check_new_password(password: &str) -> Result<(), (StatusCode, String)> {
    let length = password.chars().count();
    if length < MIN_PASSWORD_LENGTH {
        return Err((StatusCode::BAD_REQUEST, format!("Password must be at least {MIN_PASSWORD_LENGTH} characters")));
    }
    if length > MAX_PASSWORD_LENGTH {
        return Err((StatusCode::BAD_REQUEST, format!("Password must be at most {MAX_PASSWORD_LENGTH} characters")));
    }
    Ok(())
}

pub async fn hash_password(password: &str) -> Result<String, String> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || hash_blocking(&password))
        .await
        .map_err(|error| error.to_string())?
}

pub async fn verify_password(password: &str, stored_hash: &str) -> bool {
    let password = password.to_string();
    let stored_hash = stored_hash.to_string();
    tokio::task::spawn_blocking(move || verify_blocking(&password, &stored_hash))
        .await
        .unwrap_or(false)
}

pub async fn verify_login(password: &str, stored_hash: Option<&str>) -> bool {
    let password = password.to_string();
    let stored_hash = stored_hash.filter(|hash| has_password(hash)).map(str::to_string);
    tokio::task::spawn_blocking(move || match stored_hash {
        Some(stored_hash) => verify_blocking(&password, &stored_hash),
        None => {
            verify_blocking(&password, &UNKNOWN_USER_HASH);
            false
        }
    })
    .await
    .unwrap_or(false)
}

pub fn has_password(stored_hash: &str) -> bool {
    !stored_hash.is_empty() && stored_hash != NO_PASSWORD
}
