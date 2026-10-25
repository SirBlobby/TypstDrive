use sqlx::AnyPool;
use uuid::Uuid;

use super::palette::{Theme, ThemeInput, ThemeModes};
use crate::db::{sql, timestamp_now};

type ThemeRow = (String, Option<String>, String, String);

fn theme_from_row((id, owner_id, name, palette): ThemeRow) -> Option<(Option<String>, Theme)> {
    let modes: ThemeModes = serde_json::from_str(&palette).ok()?;
    Some((owner_id, Theme { id, name, modes }))
}

fn encode(input: &ThemeInput) -> String {
    serde_json::to_string(&input.modes).unwrap_or_default()
}

pub async fn site_themes(db: &AnyPool) -> Result<Vec<Theme>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ThemeRow>(sql(
        "SELECT id, owner_id, name, palette FROM themes WHERE owner_id IS NULL ORDER BY created_at ASC",
    ))
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().filter_map(theme_from_row).map(|(_, theme)| theme).collect())
}

pub async fn personal_themes(db: &AnyPool, user_id: &str) -> Result<Vec<Theme>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ThemeRow>(sql(
        "SELECT id, owner_id, name, palette FROM themes WHERE owner_id = $1 ORDER BY created_at ASC",
    ))
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().filter_map(theme_from_row).map(|(_, theme)| theme).collect())
}

pub async fn all_personal_themes(db: &AnyPool) -> Result<Vec<(String, Theme)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ThemeRow>(sql(
        "SELECT id, owner_id, name, palette FROM themes WHERE owner_id IS NOT NULL ORDER BY created_at ASC",
    ))
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(theme_from_row)
        .filter_map(|(owner_id, theme)| Some((owner_id?, theme)))
        .collect())
}

pub async fn insert(db: &AnyPool, owner_id: Option<&str>, input: &ThemeInput) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(sql("INSERT INTO themes (id, owner_id, name, palette) VALUES ($1, $2, $3, $4)"))
        .bind(&id)
        .bind(owner_id.map(str::to_string))
        .bind(&input.name)
        .bind(encode(input))
        .execute(db)
        .await?;
    Ok(id)
}

pub async fn update(db: &AnyPool, id: &str, owner_id: Option<&str>, input: &ThemeInput) -> Result<(), sqlx::Error> {
    let query = match owner_id {
        Some(_) => sql("UPDATE themes SET name = $1, palette = $2, updated_at = $3 WHERE id = $4 AND owner_id = $5"),
        None => sql("UPDATE themes SET name = $1, palette = $2, updated_at = $3 WHERE id = $4 AND owner_id IS NULL"),
    };
    let mut statement = sqlx::query(query).bind(&input.name).bind(encode(input)).bind(timestamp_now()).bind(id);
    if let Some(owner_id) = owner_id {
        statement = statement.bind(owner_id);
    }
    statement.execute(db).await?;
    Ok(())
}

pub async fn delete(db: &AnyPool, id: &str, owner_id: Option<&str>) -> Result<(), sqlx::Error> {
    let query = match owner_id {
        Some(_) => sql("DELETE FROM themes WHERE id = $1 AND owner_id = $2"),
        None => sql("DELETE FROM themes WHERE id = $1 AND owner_id IS NULL"),
    };
    let mut statement = sqlx::query(query).bind(id);
    if let Some(owner_id) = owner_id {
        statement = statement.bind(owner_id);
    }
    statement.execute(db).await?;
    Ok(())
}
