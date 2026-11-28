use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use std::net::SocketAddr;

use axum::extract::ConnectInfo;

use crate::accounts::{client_info, cookies, sessions};
use crate::compile::limits;
use crate::AppState;

const SESSION_AWARE_PREFIXES: &[&str] = &["/api/", "/yjs/", "/v1/"];

pub async fn attach_request_context(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let socket = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|info| info.0);
    let address_owner = format!("address:{}", client_info::client_ip(request.headers(), socket));
    let path = request.uri().path();
    if !SESSION_AWARE_PREFIXES.iter().any(|prefix| path.starts_with(prefix)) {
        return limits::for_owner(address_owner, next.run(request)).await;
    }

    let session = sessions::resolve(&state, request.headers()).await;
    let session_user_id = session.as_ref().map(|session| session.user_id.clone());
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

    let compile_owner = session_user_id.map_or(address_owner, |user_id| format!("user:{user_id}"));
    let grants = crate::sharing::links::grants_from_headers(&state.key, request.headers());
    limits::for_owner(compile_owner, crate::sharing::links::with_grants(grants, next.run(request))).await
}
