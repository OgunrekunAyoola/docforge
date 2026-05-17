pub mod auth;
pub mod health;
pub mod jobs;
pub mod middleware;

use crate::{config::Config, ollama::OllamaClient, queue::RedisPool};
use axum::http::{HeaderName, HeaderValue};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use middleware::rate_limit::IpRateLimiter;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    #[allow(dead_code)]
    pub ollama: Arc<OllamaClient>,
    pub metrics: PrometheusHandle,
}

pub fn create_router(state: AppState) -> Router {
    let max_body = state.config.max_file_size_bytes();

    let auth_limiter = IpRateLimiter::per_second(10);
    let auth_routes = Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .layer(axum_middleware::from_fn(move |req, next| {
            middleware::rate_limit::by_ip(Arc::clone(&auth_limiter), req, next)
        }));

    let protected = Router::new()
        .route("/api/jobs", post(jobs::create_job).get(jobs::list_jobs))
        .route("/api/jobs/:id", get(jobs::get_job).delete(jobs::delete_job))
        .route(
            "/api/jobs/:id/documents",
            post(jobs::upload_documents).get(jobs::list_documents),
        )
        .route("/api/jobs/:id/result", get(jobs::get_result))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    let public = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/metrics", get(health::metrics));

    Router::new()
        .merge(protected)
        .merge(auth_routes)
        .merge(public)
        .layer(RequestBodyLimitLayer::new(max_body))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(
            CorsLayer::new()
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::DELETE,
                ])
                .allow_headers(Any)
                .allow_origin(Any),
        )
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .with_state(state)
}
