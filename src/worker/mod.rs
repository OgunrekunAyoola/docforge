pub mod transformer;

use crate::{
    db,
    ollama::OllamaClient,
    queue::{
        consumer::{dequeue_job, send_to_dead_letter, MAX_ATTEMPTS},
        producer::enqueue_retry,
        RedisPool,
    },
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn run(pool: PgPool, redis: RedisPool, ollama: Arc<OllamaClient>, concurrency: usize) {
    let semaphore = Arc::new(Semaphore::new(concurrency));

    tracing::info!(concurrency, "worker started");

    loop {
        let msg = match dequeue_job(&redis).await {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(error = %e, "failed to dequeue job, retrying in 5s");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let permit = Arc::clone(&semaphore)
            .acquire_owned()
            .await
            // Safety: semaphore is never closed
            .expect("semaphore closed");

        let pool = pool.clone();
        let redis = redis.clone();
        let ollama = Arc::clone(&ollama);

        tokio::spawn(async move {
            let _permit = permit;
            let job_id = msg.job_id;

            if let Err(e) = transformer::process_job(&pool, &ollama, job_id).await {
                tracing::error!(%job_id, error = %e, attempt = msg.attempt, "job processing failed");

                if msg.attempt >= MAX_ATTEMPTS {
                    let _ = db::jobs::update_status(&pool, job_id, "failed", Some(&e.to_string()))
                        .await;
                    let _ = send_to_dead_letter(&redis, &msg).await;
                } else {
                    let _ = enqueue_retry(&redis, msg).await;
                }
            }
        });
    }
}
