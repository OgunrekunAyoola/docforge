use super::{RedisPool, QUEUE_KEY};
use anyhow::Result;
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueMessage {
    pub job_id: Uuid,
    pub enqueued_at: chrono::DateTime<Utc>,
    pub attempt: u32,
}

#[tracing::instrument(skip(pool))]
pub async fn enqueue_job(pool: &RedisPool, job_id: Uuid) -> Result<()> {
    let msg = QueueMessage {
        job_id,
        enqueued_at: Utc::now(),
        attempt: 1,
    };
    let payload = serde_json::to_string(&msg)?;

    let mut conn = pool.get().await?;
    conn.rpush::<_, _, ()>(QUEUE_KEY, payload).await?;

    tracing::info!(%job_id, "job enqueued");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn enqueue_retry(pool: &RedisPool, mut msg: QueueMessage) -> Result<()> {
    msg.attempt += 1;
    let payload = serde_json::to_string(&msg)?;
    let delay_secs = retry_delay(msg.attempt);

    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;

    let mut conn = pool.get().await?;
    conn.rpush::<_, _, ()>(QUEUE_KEY, payload).await?;

    tracing::info!(job_id = %msg.job_id, attempt = msg.attempt, "job requeued for retry");
    Ok(())
}

pub fn retry_delay(attempt: u32) -> u64 {
    match attempt {
        2 => 30,
        3 => 300,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_message_serializes_roundtrip() {
        let id = Uuid::new_v4();
        let msg = QueueMessage {
            job_id: id,
            enqueued_at: Utc::now(),
            attempt: 1,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: QueueMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.job_id, id);
        assert_eq!(decoded.attempt, 1);
    }

    #[test]
    fn retry_delay_values() {
        assert_eq!(retry_delay(1), 0);
        assert_eq!(retry_delay(2), 30);
        assert_eq!(retry_delay(3), 300);
        assert_eq!(retry_delay(4), 0);
    }
}
