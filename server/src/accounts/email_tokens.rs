use chrono::{TimeDelta, Utc};
use uuid::Uuid;

use crate::accounts::sessions::format_time;
use crate::db::sql;
use crate::AppState;

const RESEND_COOLDOWN_SECONDS: i64 = 60;
pub const SIGN_IN_CODE_MINUTES: i64 = 10;
pub const VERIFY_EMAIL_CODE_MINUTES: i64 = 30;
const CODE_DIGITS: u32 = 6;

#[derive(Clone, Copy)]
pub enum TokenPurpose {
    VerifyEmail,
    ResetPassword,
    SignInCode,
    VerifyEmailCode,
}

impl TokenPurpose {
    fn as_str(self) -> &'static str {
        match self {
            Self::VerifyEmail => "verify_email",
            Self::ResetPassword => "reset_password",
            Self::SignInCode => "sign_in_code",
            Self::VerifyEmailCode => "verify_email_code",
        }
    }

    fn lifetime(self) -> TimeDelta {
        match self {
            Self::VerifyEmail => TimeDelta::hours(48),
            Self::ResetPassword => TimeDelta::hours(1),
            Self::SignInCode => TimeDelta::minutes(SIGN_IN_CODE_MINUTES),
            Self::VerifyEmailCode => TimeDelta::minutes(VERIFY_EMAIL_CODE_MINUTES),
        }
    }
}

pub struct ConsumedToken {
    pub user_id: String,
    pub email: String,
}

fn hash_token(token: &str) -> String {
    crate::hashing::sha256_hex(token.as_bytes())
}

fn hash_code(user_id: &str, code: &str, purpose: TokenPurpose) -> String {
    crate::hashing::sha256_hex(format!("{}:{user_id}:{code}", purpose.as_str()).as_bytes())
}

fn generate_code() -> String {
    let range = 10u128.pow(CODE_DIGITS);
    let number = Uuid::new_v4().as_u128() % range;
    format!("{number:0width$}", width = CODE_DIGITS as usize)
}

pub async fn recently_issued(state: &AppState, user_id: &str, purpose: TokenPurpose) -> bool {
    let cutoff = format_time(Utc::now() - TimeDelta::seconds(RESEND_COOLDOWN_SECONDS));
    sqlx::query_as::<_, (i64,)>(sql(
        "SELECT CAST(COUNT(*) AS BIGINT) FROM email_tokens WHERE user_id = $1 AND purpose = $2 AND created_at > $3",
    ))
    .bind(user_id)
    .bind(purpose.as_str())
    .bind(cutoff)
    .fetch_one(&state.db)
    .await
    .map(|(count,)| count > 0)
    .unwrap_or(false)
}

pub async fn issue(state: &AppState, user_id: &str, email: &str, purpose: TokenPurpose) -> Result<String, sqlx::Error> {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    store(state, user_id, email, purpose, &hash_token(&token)).await?;
    Ok(token)
}

pub async fn issue_code(state: &AppState, user_id: &str, email: &str, purpose: TokenPurpose) -> Result<String, sqlx::Error> {
    let code = generate_code();
    store(state, user_id, email, purpose, &hash_code(user_id, &code, purpose)).await?;
    Ok(code)
}

async fn store(state: &AppState, user_id: &str, email: &str, purpose: TokenPurpose, token_hash: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    sqlx::query(sql("DELETE FROM email_tokens WHERE (user_id = $1 AND purpose = $2) OR expires_at < $3"))
        .bind(user_id)
        .bind(purpose.as_str())
        .bind(format_time(now))
        .execute(&state.db)
        .await?;

    sqlx::query(sql(
        "INSERT INTO email_tokens (id, user_id, token_hash, purpose, email, created_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    ))
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(token_hash)
    .bind(purpose.as_str())
    .bind(email)
    .bind(format_time(now))
    .bind(format_time(now + purpose.lifetime()))
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn consume(state: &AppState, token: &str, purpose: TokenPurpose) -> Option<ConsumedToken> {
    consume_hash(state, &hash_token(token.trim()), purpose).await
}

pub async fn consume_code(state: &AppState, user_id: &str, code: &str, purpose: TokenPurpose) -> Option<ConsumedToken> {
    let digits: String = code.chars().filter(char::is_ascii_digit).collect();
    if digits.len() != CODE_DIGITS as usize {
        return None;
    }
    consume_hash(state, &hash_code(user_id, &digits, purpose), purpose).await
}

async fn consume_hash(state: &AppState, token_hash: &str, purpose: TokenPurpose) -> Option<ConsumedToken> {
    let (id, user_id, email, expires_at) = sqlx::query_as::<_, (String, String, String, String)>(sql(
        "SELECT id, user_id, email, expires_at FROM email_tokens WHERE token_hash = $1 AND purpose = $2",
    ))
    .bind(token_hash)
    .bind(purpose.as_str())
    .fetch_optional(&state.db)
    .await
    .ok()??;

    let _ = sqlx::query(sql("DELETE FROM email_tokens WHERE id = $1"))
        .bind(&id)
        .execute(&state.db)
        .await;

    if expires_at < format_time(Utc::now()) {
        return None;
    }
    Some(ConsumedToken { user_id, email })
}
