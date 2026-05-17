mod api;
mod config;
mod db;
mod error;
mod models;
mod observability;
mod ollama;
mod queue;
mod worker;

use anyhow::Result;
use api::AppState;
use config::Config;
use ollama::OllamaClient;
use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let worker_mode = args.iter().any(|a| a == "--worker");
    let migrate_only = args.iter().any(|a| a == "--migrate-only");

    let cfg = Config::from_env()?;
    observability::init_tracing()?;

    let db_pool = db::create_pool(&cfg.database_url).await?;
    db::run_migrations(&db_pool).await?;
    tracing::info!("database migrations applied");

    if migrate_only {
        tracing::info!("migrate-only mode, exiting");
        return Ok(());
    }

    let metrics_handle = observability::metrics::init_prometheus()?;

    tracing::info!(
        mode = if worker_mode { "worker" } else { "api" },
        version = env!("CARGO_PKG_VERSION"),
        "docforge starting"
    );

    let redis_pool = queue::create_pool(&cfg.redis_url)?;
    tracing::info!("redis pool created");

    let ollama = Arc::new(OllamaClient::new(
        cfg.llm_base_url.clone(),
        cfg.llm_model.clone(),
        cfg.llm_api_key.clone(),
        120,
    ));

    let state = AppState {
        db: db_pool.clone(),
        redis: redis_pool.clone(),
        config: Arc::new(cfg.clone()),
        ollama: Arc::clone(&ollama),
        metrics: metrics_handle,
    };

    if worker_mode {
        worker::run(db_pool, redis_pool, ollama, cfg.worker_concurrency).await;
        return Ok(());
    }

    // Boot worker in background alongside API server
    {
        let db = db_pool.clone();
        let redis = redis_pool.clone();
        let oll = Arc::clone(&ollama);
        let concurrency = cfg.worker_concurrency;
        tokio::spawn(async move {
            worker::run(db, redis, oll, concurrency).await;
        });
    }

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let router = api::create_router(state);

    tracing::info!(%addr, "API server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("shutdown signal received");
}
