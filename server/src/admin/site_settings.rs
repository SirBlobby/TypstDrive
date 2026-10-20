use std::collections::HashMap;

use crate::AppState;
use crate::db::sql;

pub async fn load_settings(state: &AppState, prefix: &str) -> HashMap<String, String> {
    sqlx::query_as::<_, (String, String)>(
        sql("SELECT setting_key, setting_value FROM site_settings WHERE setting_key LIKE $1")
    )
    .bind(format!("{prefix}%"))
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect()
}

pub async fn save_settings(state: &AppState, entries: &[(&str, String)]) -> Result<(), sqlx::Error> {
    let mut transaction = state.db.begin().await?;
    for (key, value) in entries {
        sqlx::query(
            sql("INSERT INTO site_settings (setting_key, setting_value) VALUES ($1, $2) \
             ON CONFLICT (setting_key) DO UPDATE SET setting_value = excluded.setting_value")
        )
        .bind(*key)
        .bind(value)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await
}
