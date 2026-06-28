use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde_json::json;

pub async fn logout(State(state): State<crate::AppState>, jar: CookieJar) -> impl IntoResponse {
    if let Some(cookie) = jar.get("BEAM_PIN") {
        state.active_sessions.write().await.remove(cookie.value());
    }
    let new_jar = jar.add(Cookie::build(("BEAM_PIN", "")).path("/").build());
    let res = (StatusCode::OK, Json(json!({ "success": true }))).into_response();
    (new_jar, res).into_response()
}
