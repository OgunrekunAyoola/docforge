use super::{producer::QueueMessage, RedisPool, DEAD_LETTER_KEY, QUEUE_KEY};
use anyhow::Result;
use deadpool_redis::redis::AsyncCommands;

pub const MAX_ATTEMPTS: u32 = 3;

#[tracing::instrument(skip(pool))]
pub async fn dequeue_job(pool: &RedisPool) -> Result<QueueMessage> {
    let mut conn = pool.get().await?;
    // BLPOP blocks until a message is available; timeout 0 = block forever
    let result: (String, String) = conn.blpop(QUEUE_KEY, 0.0).await?;
    let msg: QueueMessage = serde_json::from_str(&result.1)?;
    Ok(msg)
}

#[tracing::instrument(skip(pool))]
pub async fn send_to_dead_letter(pool: &RedisPool, msg: &QueueMessage) -> Result<()> {
    let payload = serde_json::to_string(msg)?;
    let mut conn = pool.get().await?;
    conn.rpush::<_, _, ()>(DEAD_LETTER_KEY, payload).await?;
    tracing::warn!(job_id = %msg.job_id, "job moved to dead letter queue");
    Ok(())
}
