use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub worker_concurrency: usize,
    pub max_file_size_mb: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .set_default("host", "0.0.0.0")?
            .set_default("port", 3000)?
            .set_default("llm_base_url", "https://api.groq.com/openai/v1")?
            .set_default("llm_model", "llama-3.3-70b-versatile")?
            .set_default("jwt_expiry_hours", 24)?
            .set_default("worker_concurrency", 3)?
            .set_default("max_file_size_mb", 10)?
            .build()?;
        Ok(cfg.try_deserialize()?)
    }

    pub fn max_file_size_bytes(&self) -> usize {
        self.max_file_size_mb * 1024 * 1024
    }
}
