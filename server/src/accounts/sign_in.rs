use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounts::client_info::ClientInfo;
use crate::accounts::email_code::CODE_ATTEMPT_RULES;
use crate::accounts::two_factor::{self, Method};
use crate::accounts::{auth, cookies, lookup, passwords, sessions};
use crate::db::sql;
use crate::models::{User, UserProfile};
use crate::AppState;

type ApiError = (StatusCode, String);

const PENDING_COOKIE: &str = "typstdrive_pending_sign_in";
const PENDING_COOKIE_PATH: &str = "/api/auth/sign-in";
const PENDING_MINUTES: i64 = 10;
const CODE_SCOPE: &str = "two_factor";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FirstStep {
    Password,
    EmailCode,
    SingleSignOn,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NextStep {
    ChangePassword,
    TwoFactor,
}

#[derive(Serialize)]
pub struct PendingView {
    next_step: NextStep,
    method: Option<Method>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SignInResult {
    SignedIn(UserProfile),
    Pending(PendingView),
}

pub enum Outcome {
    SignedIn(Cookie<'static>),
    Pending(Cookie<'static>, PendingView),
}

struct PendingSignIn {
    id: String,
    user_id: String,
    change_password: bool,
    second_factor: Option<Method>,
}

fn view_for(change_password: bool, second_factor: Option<Method>) -> PendingView {
    let next_step = if change_password { NextStep::ChangePassword } else { NextStep::TwoFactor };
    PendingView { next_step, method: second_factor }
}

impl PendingSignIn {
    fn view(&self) -> PendingView {
        view_for(self.change_password, self.second_factor)
    }
}

#[derive(Deserialize)]
pub struct NewPasswordInput {
    new_password: String,
}

#[derive(Deserialize)]
pub struct CodeInput {
    code: String,
}

fn hash_token(token: &str) -> String {
    crate::hashing::sha256_hex(token.as_bytes())
}

fn pending_cookie(token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(PENDING_COOKIE, token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path(PENDING_COOKIE_PATH);
    cookie.set_secure(crate::site_url::uses_https());
    cookie.set_max_age(cookie::time::Duration::minutes(PENDING_MINUTES));
    cookie
}

fn cleared_pending_cookie() -> Cookie<'static> {
    Cookie::build(PENDING_COOKIE).path(PENDING_COOKIE_PATH).build()
}

fn expired() -> ApiError {
    (StatusCode::UNAUTHORIZED, "This sign-in has expired. Sign in again.".to_string())
}

async fn send_code_quietly(state: &AppState, user_id: &str) {
    if let Err((_, message)) = two_factor::send_email_code_to_user(state, user_id).await {
        tracing::warn!("Could not send the two-factor email code: {message}");
    }
}

pub async fn begin(state: &AppState, user: &User, first_step: FirstStep, change_password: bool, client: &ClientInfo) -> Result<Outcome, ApiError> {
    let method = two_factor::method_of(state, &user.id).await;
    let second_factor = method.filter(|method| !(*method == Method::Email && first_step == FirstStep::EmailCode));
    if !change_password && second_factor.is_none() {
        return sessions::start_session(state, &user.id, client).await.map(Outcome::SignedIn);
    }

    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let now = Utc::now();
    sqlx::query(sql("DELETE FROM pending_sign_ins WHERE user_id = $1 OR expires_at < $2"))
        .bind(&user.id)
        .bind(sessions::format_time(now))
        .execute(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?;
    sqlx::query(sql(
        "INSERT INTO pending_sign_ins (id, user_id, token_hash, change_password, second_factor, expires_at) VALUES ($1, $2, $3, $4, $5, $6)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(&user.id)
    .bind(hash_token(&token))
    .bind(if change_password { 1i64 } else { 0i64 })
    .bind(second_factor.map(Method::as_str))
    .bind(sessions::format_time(now + TimeDelta::minutes(PENDING_MINUTES)))
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;

    if !change_password && second_factor == Some(Method::Email) {
        send_code_quietly(state, &user.id).await;
    }
    Ok(Outcome::Pending(pending_cookie(token), view_for(change_password, second_factor)))
}

pub async fn respond(state: &AppState, jar: CookieJar, user_id: &str, outcome: Outcome) -> Result<(CookieJar, Json<SignInResult>), ApiError> {
    match outcome {
        Outcome::SignedIn(cookie) => {
            let user = lookup::user_by_id(state, user_id).await?.ok_or(expired())?;
            let profile = auth::profile_for(state, user).await;
            Ok((jar.add(cookie).remove(cleared_pending_cookie()), Json(SignInResult::SignedIn(profile))))
        }
        Outcome::Pending(cookie, view) => Ok((jar.add(cookie), Json(SignInResult::Pending(view)))),
    }
}

pub async fn complete(
    state: &AppState,
    jar: CookieJar,
    user: &User,
    first_step: FirstStep,
    change_password: bool,
    client: &ClientInfo,
) -> Result<(CookieJar, Json<SignInResult>), ApiError> {
    let outcome = begin(state, user, first_step, change_password, client).await?;
    respond(state, jar, &user.id, outcome).await
}

async fn load_pending(state: &AppState, headers: &HeaderMap) -> Result<PendingSignIn, ApiError> {
    let token = cookies::read(headers, PENDING_COOKIE).ok_or(expired())?;
    let (id, user_id, change_password, second_factor) = sqlx::query_as::<_, (String, String, i64, Option<String>)>(sql(
        "SELECT id, user_id, change_password, second_factor FROM pending_sign_ins WHERE token_hash = $1 AND expires_at > $2",
    ))
    .bind(hash_token(&token))
    .bind(sessions::format_time(Utc::now()))
    .fetch_optional(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?
    .ok_or(expired())?;
    Ok(PendingSignIn {
        id,
        user_id,
        change_password: change_password != 0,
        second_factor: second_factor.as_deref().and_then(Method::parse),
    })
}

async fn finish(state: &AppState, jar: CookieJar, pending: &PendingSignIn, client: &ClientInfo) -> Result<(CookieJar, Json<SignInResult>), ApiError> {
    let removed = sqlx::query(sql("DELETE FROM pending_sign_ins WHERE id = $1"))
        .bind(&pending.id)
        .execute(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?;
    if removed.rows_affected() == 0 {
        return Err(expired());
    }
    let cookie = sessions::start_session(state, &pending.user_id, client).await?;
    respond(state, jar, &pending.user_id, Outcome::SignedIn(cookie)).await
}

pub async fn pending_status(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<PendingView>, ApiError> {
    Ok(Json(load_pending(&state, &headers).await?.view()))
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    client: ClientInfo,
    jar: CookieJar,
    Json(payload): Json<NewPasswordInput>,
) -> Result<(CookieJar, Json<SignInResult>), ApiError> {
    let pending = load_pending(&state, &headers).await?;
    if !pending.change_password {
        return Err((StatusCode::BAD_REQUEST, "No new password is needed".to_string()));
    }
    passwords::check_account_password(&state, &payload.new_password).await?;
    let password_hash = passwords::hash_password(&payload.new_password)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    sqlx::query(sql(
        "UPDATE users SET password_hash = $1, password_change_required = 0, temporary_password_expires_at = NULL WHERE id = $2",
    ))
    .bind(&password_hash)
    .bind(&pending.user_id)
    .execute(&state.db)
    .await
    .map_err(crate::server_errors::hidden)?;

    if pending.second_factor.is_none() {
        return finish(&state, jar, &pending, &client).await;
    }
    sqlx::query(sql("UPDATE pending_sign_ins SET change_password = 0 WHERE id = $1"))
        .bind(&pending.id)
        .execute(&state.db)
        .await
        .map_err(crate::server_errors::hidden)?;
    if pending.second_factor == Some(Method::Email) {
        send_code_quietly(&state, &pending.user_id).await;
    }
    Ok((jar, Json(SignInResult::Pending(view_for(false, pending.second_factor)))))
}

pub async fn verify_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    client: ClientInfo,
    jar: CookieJar,
    Json(payload): Json<CodeInput>,
) -> Result<(CookieJar, Json<SignInResult>), ApiError> {
    let pending = load_pending(&state, &headers).await?;
    if pending.change_password || pending.second_factor.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Choose a new password first".to_string()));
    }
    state.login_guard.ensure_allowed(CODE_SCOPE, &pending.user_id, &client.ip_address)?;
    if !two_factor::accepts_code(&state, &pending.user_id, &payload.code).await {
        state.login_guard.record_failure(CODE_SCOPE, &pending.user_id, &client.ip_address, CODE_ATTEMPT_RULES);
        return Err((StatusCode::UNAUTHORIZED, "The code is wrong or has expired".to_string()));
    }
    state.login_guard.record_success(CODE_SCOPE, &pending.user_id);
    finish(&state, jar, &pending, &client).await
}

pub async fn resend_email_code(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    let pending = load_pending(&state, &headers).await?;
    if pending.change_password || pending.second_factor != Some(Method::Email) {
        return Err((StatusCode::BAD_REQUEST, "This sign-in does not use email codes".to_string()));
    }
    two_factor::send_email_code_to_user(&state, &pending.user_id).await?;
    Ok(StatusCode::ACCEPTED)
}
