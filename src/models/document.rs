use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub job_id: Uuid,
    pub filename: String,
    pub content: String,
    pub doc_type: String,
    pub created_at: DateTime<Utc>,
}
