// TODO: switch to sqlx::query_as! macros once DATABASE_URL is available (requires live Postgres)
use crate::{error::AppError, models::job::Job};
use sqlx::PgPool;
use uuid::Uuid;

const SELECT: &str =
    "id, user_id, status, name, source_context, target_context, created_at, updated_at, completed_at, error";

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    source_context: &str,
    target_context: &str,
) -> Result<Job, AppError> {
    let job = sqlx::query_as::<_, Job>(&format!(
        "INSERT INTO jobs (user_id, name, source_context, target_context)
         VALUES ($1, $2, $3, $4)
         RETURNING {SELECT}"
    ))
    .bind(user_id)
    .bind(name)
    .bind(source_context)
    .bind(target_context)
    .fetch_one(pool)
    .await?;

    Ok(job)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<Job>, AppError> {
    let job = sqlx::query_as::<_, Job>(&format!(
        "SELECT {SELECT} FROM jobs WHERE id = $1 AND user_id = $2"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

pub async fn find_by_id_any_user(pool: &PgPool, id: Uuid) -> Result<Option<Job>, AppError> {
    let job = sqlx::query_as::<_, Job>(&format!(
        "SELECT {SELECT} FROM jobs WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Job>, AppError> {
    let jobs = sqlx::query_as::<_, Job>(&format!(
        "SELECT {SELECT} FROM jobs WHERE user_id = $1
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(jobs)
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE jobs
         SET status = $2,
             error = $3,
             updated_at = NOW(),
             completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(error)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM jobs WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
