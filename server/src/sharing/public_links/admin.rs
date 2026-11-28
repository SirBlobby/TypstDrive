use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Serialize;

use super::policy::{load_policy, PublicLinkPolicy};
use super::{all_links, cache, internal_error, ApiError, PublicLinkView};
use crate::accounts::client_info::ClientInfo;
use crate::admin::users::require_admin;
use crate::db::sql;
use crate::AppState;

#[derive(Serialize)]
pub struct PublicLinkOverview {
    pub policy: PublicLinkPolicy,
    pub links: Vec<PublicLinkView>,
}

pub async fn overview(State(state): State<AppState>, jar: SignedCookieJar) -> Result<Json<PublicLinkOverview>, ApiError> {
    require_admin(&state, &jar).await?;
    Ok(Json(PublicLinkOverview { policy: load_policy(&state).await, links: all_links(&state).await? }))
}

pub async fn delete_link(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
    jar: SignedCookieJar,
    client: ClientInfo,
) -> Result<StatusCode, ApiError> {
    let admin_id = require_admin(&state, &jar).await?;
    let slug = sqlx::query_as::<_, (String,)>(sql("SELECT slug FROM public_links WHERE id = $1"))
        .bind(&link_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?
        .map(|(slug,)| slug)
        .ok_or((StatusCode::NOT_FOUND, "Public link not found".to_string()))?;

    sqlx::query(sql("DELETE FROM public_links WHERE id = $1"))
        .bind(&link_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    cache::forget(&link_id);

    crate::admin::audit::record(&state, &admin_id, &client, "public_links.delete", &slug, "").await;
    Ok(StatusCode::NO_CONTENT)
}
