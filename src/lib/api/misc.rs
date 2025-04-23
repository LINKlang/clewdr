use axum::{extract::{Path, State}, Json};
use axum_auth::AuthBearer;
use rquest::StatusCode;
use tracing::{error, info, warn};

use crate::{
    VERSION_INFO, config::CookieStatus, cookie_manager::CookieStatusInfo, state::ClientState, error::ClewdrError,
};

pub async fn api_submit(
    State(s): State<ClientState>,
    AuthBearer(t): AuthBearer,
    Json(mut c): Json<CookieStatus>,
) -> StatusCode {
    if !s.config.auth(&t) {
        return StatusCode::UNAUTHORIZED;
    }
    if !c.cookie.validate() {
        warn!("Invalid cookie: {}", c.cookie);
        return StatusCode::BAD_REQUEST;
    }
    c.reset_time = None;

    match s.event_sender.submit(c).await {
        Ok(_) => StatusCode::CREATED,
        Err(ClewdrError::DuplicateCookie) => StatusCode::CONFLICT,
        Err(e) => {
            error!("Failed to submit cookie: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn api_get_cookies(
    State(s): State<ClientState>,
    AuthBearer(t): AuthBearer,
) -> Result<Json<CookieStatusInfo>, (StatusCode, Json<serde_json::Value>)> {
    if !s.config.auth(&t) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized"
            })),
        ));
    }

    match s.event_sender.get_status().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get cookie status: {}", e)
            })),
        )),
    }
}

pub async fn api_delete_cookie(
    State(s): State<ClientState>,
    AuthBearer(t): AuthBearer,
    Path(cookie_string): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !s.config.auth(&t) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized"
            })),
        ));
    }

    // Convert string to CookieStatus
    let cookie = CookieStatus::new(&cookie_string, None);

    match s.event_sender.delete_cookie(cookie).await {
        Ok(_) => {
            info!("Cookie deleted successfully: {}", cookie_string);
            Ok(StatusCode::NO_CONTENT)
        },
        Err(e) => {
            error!("Failed to delete cookie: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to delete cookie: {}", e)
                })),
            ))
        },
    }
}

pub async fn api_version() -> String {
    VERSION_INFO.to_string()
}

pub async fn api_auth(State(s): State<ClientState>, AuthBearer(t): AuthBearer) -> StatusCode {
    if !s.config.auth(&t) {
        return StatusCode::UNAUTHORIZED;
    }
    info!("Auth token accepted,");
    StatusCode::OK
}
