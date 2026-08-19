use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use defs::ApiKeyRole;
use serde_json::json;

use crate::AppState;

const API_KEY_HEADER: &str = "api-key";

fn extract_key(req: &Request) -> Option<&str> {
    req.headers().get(API_KEY_HEADER)?.to_str().ok()
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Missing or invalid api-key header" })),
    )
        .into_response()
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "This api-key does not have write access" })),
    )
        .into_response()
}

pub async fn require_write_key(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let Some(key) = extract_key(&req) else {
        return unauthorized();
    };

    match state.keys.find(key) {
        Some(entry) if entry.role == ApiKeyRole::ReadWrite => next.run(req).await,
        Some(_) => forbidden(),
        None => unauthorized(),
    }
}

pub async fn require_read_key(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let Some(key) = extract_key(&req) else {
        return unauthorized();
    };

    match state.keys.find(key) {
        Some(_) => next.run(req).await,
        None => unauthorized(),
    }
}
