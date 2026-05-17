// TODO: switch to sqlx::query_as! macros once DATABASE_URL is available (requires live Postgres)
use crate::{error::AppError, models::document::Document};
use sqlx::PgPool;
use uuid::Uuid;

pub struct NewDocument {
    pub job_id: Uuid,
    pub filename: String,
    pub content: String,
    pub doc_type: String,
}

pub async fn create_batch(
    pool: &PgPool,
    docs: Vec<NewDocument>,
) -> Result<Vec<Document>, AppError> {
    let mut results = Vec::with_capacity(docs.len());

    for doc in docs {
        let document = sqlx::query_as::<_, Document>(
            "INSERT INTO documents (job_id, filename, content, doc_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id, job_id, filename, content, doc_type, created_at",
        )
        .bind(doc.job_id)
        .bind(&doc.filename)
        .bind(&doc.content)
        .bind(&doc.doc_type)
        .fetch_one(pool)
        .await?;

        results.push(document);
    }

    Ok(results)
}

pub async fn find_by_job(
    pool: &PgPool,
    job_id: Uuid,
    doc_type: &str,
) -> Result<Vec<Document>, AppError> {
    let docs = sqlx::query_as::<_, Document>(
        "SELECT id, job_id, filename, content, doc_type, created_at
         FROM documents
         WHERE job_id = $1 AND doc_type = $2
         ORDER BY created_at ASC",
    )
    .bind(job_id)
    .bind(doc_type)
    .fetch_all(pool)
    .await?;

    Ok(docs)
}
