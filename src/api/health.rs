use crate::{api::AppState, error::AppError};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

#[tracing::instrument(skip_all)]
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

#[tracing::instrument(skip(state))]
pub async fn ready(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("database not ready".into()))?;

    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|_| AppError::BadRequest("redis not ready".into()))?;

    let _: String = deadpool_redis::redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .map_err(|_| AppError::BadRequest("redis not ready".into()))?;

    Ok((StatusCode::OK, Json(json!({ "status": "ready" }))))
}

#[tracing::instrument(skip(state))]
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.render();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        body,
    )
}
