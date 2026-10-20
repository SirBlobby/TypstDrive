use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::accounts::{cookies, sessions};
use crate::AppState;

const SESSION_AWARE_PREFIXES: &[&str] = &["/api/", "/yjs/", "/v1/"];

pub async fn attach_request_context(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    if !SESSION_AWARE_PREFIXES.iter().any(|prefix| path.starts_with(prefix)) {
        return next.run(request).await;
    }

    let session = sessions::resolve(&state, request.headers()).await;
    let acting_user_id = match &session {
        Some(session) => {
            let acting = crate::teams::workspace::acting_account(
                &state,
                request.method(),
                request.uri(),
                request.headers(),
                &session.user_id,
            )
            .await;
            match acting {
                Ok(account_id) => Some(account_id),
                Err(rejection) => return rejection.into_response(),
            }
        }
        None => None,
    };

    cookies::replace_signed(request.headers_mut(), &state.key, cookies::USER_ID_COOKIE, acting_user_id.as_deref());
    if let Some(session) = session {
        request.extensions_mut().insert(session);
    }

    let headers = request.headers().clone();
    crate::sharing::links::with_grants(&state.key, &headers, next.run(request)).await
}
