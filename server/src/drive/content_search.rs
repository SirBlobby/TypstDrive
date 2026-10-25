use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::accounts::signed_in::SignedInUser;
use crate::db::sql;
use crate::drive::projects::decode_text_blob;
use crate::AppState;

type ApiError = (StatusCode, String);

const MINIMUM_QUERY_LENGTH: usize = 2;
const MAXIMUM_RESULTS: usize = 50;
const SNIPPET_CHARACTERS_BEFORE: usize = 60;
const SNIPPET_CHARACTERS_AFTER: usize = 140;

#[derive(Deserialize)]
pub struct ContentSearchQuery {
    pub q: String,
    pub language: Option<String>,
    pub shared: Option<bool>,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl ContentSearchQuery {
    fn accepts(&self, owner_id: &str, user_id: &str, language: Option<&str>, updated_at: &str) -> bool {
        let document_language = language.unwrap_or(crate::compile::latex::TYPST_LANGUAGE);
        let updated_day = updated_at.get(..10).unwrap_or(updated_at);
        self.language.as_deref().is_none_or(|wanted| wanted == document_language)
            && (self.shared != Some(true) || owner_id != user_id)
            && self.before.as_deref().is_none_or(|day| updated_day < day)
            && self.after.as_deref().is_none_or(|day| updated_day > day)
    }
}

#[derive(Serialize)]
pub struct ContentSearchResult {
    pub id: String,
    pub title: String,
    pub folder_id: Option<String>,
    pub updated_at: String,
    pub language: Option<String>,
    pub shared: bool,
    pub match_count: usize,
    pub snippet: String,
}

type DocumentRow = (String, String, String, Option<String>, String, Option<String>, Option<Vec<u8>>);

pub async fn search_document_text(
    State(state): State<AppState>,
    Query(query): Query<ContentSearchQuery>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<Vec<ContentSearchResult>>, ApiError> {
    let needle = query.q.trim().to_lowercase();
    if needle.chars().count() < MINIMUM_QUERY_LENGTH {
        return Ok(Json(Vec::new()));
    }

    let rows = sqlx::query_as::<_, DocumentRow>(sql(
        "SELECT d.id, d.owner_id, d.title, d.folder_id, d.updated_at, d.language, d.content FROM documents d \
         WHERE (d.owner_id = $1 OR d.id IN (SELECT c.document_id FROM collaborators c WHERE c.user_id = $2)) \
         AND d.deleted_at IS NULL ORDER BY d.updated_at DESC",
    ))
    .bind(&user_id)
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
    .into_iter()
    .filter(|(_, owner_id, _, _, updated_at, language, _)| query.accepts(owner_id, &user_id, language.as_deref(), updated_at))
    .collect::<Vec<_>>();

    let results = tokio::task::spawn_blocking(move || find_matches(rows, &needle, &user_id))
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(results))
}

fn find_matches(rows: Vec<DocumentRow>, needle: &str, user_id: &str) -> Vec<ContentSearchResult> {
    let mut results = Vec::new();
    for (id, owner_id, title, folder_id, updated_at, language, content) in rows {
        let text = decode_text_blob(content.as_deref().unwrap_or_default());
        let lowercase_text = text.to_lowercase();
        let Some(first_match) = lowercase_text.find(needle) else {
            continue;
        };

        results.push(ContentSearchResult {
            id,
            title,
            folder_id,
            updated_at,
            language,
            shared: owner_id != user_id,
            match_count: lowercase_text.matches(needle).count(),
            snippet: snippet_around(&text, &lowercase_text, first_match, needle.chars().count()),
        });
        if results.len() >= MAXIMUM_RESULTS {
            break;
        }
    }
    results
}

fn snippet_around(text: &str, lowercase_text: &str, match_byte_index: usize, needle_length: usize) -> String {
    let match_character_index = lowercase_text[..match_byte_index].chars().count();
    let characters: Vec<char> = text.chars().collect();
    let start = match_character_index.saturating_sub(SNIPPET_CHARACTERS_BEFORE);
    let end = (match_character_index + needle_length + SNIPPET_CHARACTERS_AFTER).min(characters.len());
    let start = start.min(end);

    let body: String = characters[start..end].iter().collect();
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let prefix = if start > 0 { "..." } else { "" };
    let suffix = if end < characters.len() { "..." } else { "" };
    format!("{prefix}{collapsed}{suffix}")
}
