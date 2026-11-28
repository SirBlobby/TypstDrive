use axum::http::StatusCode;
use uuid::Uuid;

use crate::accounts::random_text::{grouped, random_text};
use crate::db::sql;
use crate::AppState;

type ApiError = (StatusCode, String);

const CODE_COUNT: usize = 10;
const CODE_LENGTH: usize = 10;

fn normalize(code: &str) -> String {
    code.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn hash_code(user_id: &str, normalized_code: &str) -> String {
    crate::hashing::sha256_hex(format!("backup:{user_id}:{normalized_code}").as_bytes())
}

pub async fn replace(state: &AppState, user_id: &str) -> Result<Vec<String>, ApiError> {
    let codes: Vec<String> = (0..CODE_COUNT).map(|_| random_text(CODE_LENGTH)).collect();
    let mut transaction = state.db.begin().await.map_err(crate::server_errors::hidden)?;
    sqlx::query(sql("DELETE FROM two_factor_backup_codes WHERE user_id = $1"))
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(crate::server_errors::hidden)?;
    for code in &codes {
        sqlx::query(sql("INSERT INTO two_factor_backup_codes (id, user_id, code_hash) VALUES ($1, $2, $3)"))
            .bind(Uuid::new_v4().to_string())
            .bind(user_id)
            .bind(hash_code(user_id, code))
            .execute(&mut *transaction)
            .await
            .map_err(crate::server_errors::hidden)?;
    }
    transaction.commit().await.map_err(crate::server_errors::hidden)?;
    Ok(codes.iter().map(|code| grouped(code, CODE_LENGTH / 2)).collect())
}

pub async fn consume(state: &AppState, user_id: &str, code: &str) -> bool {
    let normalized = normalize(code);
    if normalized.len() != CODE_LENGTH {
        return false;
    }
    sqlx::query(sql("DELETE FROM two_factor_backup_codes WHERE user_id = $1 AND code_hash = $2"))
        .bind(user_id)
        .bind(hash_code(user_id, &normalized))
        .execute(&state.db)
        .await
        .is_ok_and(|result| result.rows_affected() == 1)
}

pub async fn remaining(state: &AppState, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>(sql("SELECT CAST(COUNT(*) AS BIGINT) FROM two_factor_backup_codes WHERE user_id = $1"))
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map(|(count,)| count)
        .unwrap_or(0)
}

pub async fn remove_all(state: &AppState, user_id: &str) -> Result<(), ApiError> {
    sqlx::query(sql("DELETE FROM two_factor_backup_codes WHERE user_id = $1"))
        .bind(user_id)
        .execute(&state.db)
        .await
        .map(|_| ())
        .map_err(crate::server_errors::hidden)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ignores_dashes_spaces_and_case() {
        assert_eq!(normalize("abcde-fgh23"), "ABCDEFGH23");
        assert_eq!(normalize(" ABCDE FGH23 "), "ABCDEFGH23");
    }
}
