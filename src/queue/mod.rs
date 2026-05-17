pub mod consumer;
pub mod producer;

use anyhow::Result;
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

pub type RedisPool = Pool;

pub fn create_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = RedisConfig::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}

pub const QUEUE_KEY: &str = "docforge:jobs";
pub const DEAD_LETTER_KEY: &str = "docforge:jobs:failed";
