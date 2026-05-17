use crate::{
    api::{middleware::auth::Claims, AppState},
    db,
    error::AppError,
    queue::producer::enqueue_job,
};
use axum::{
    body::Bytes,
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

const MAX_FILES_PER_JOB: usize = 20;
const ALLOWED_EXTENSIONS: &[&str] = &["md", "txt", "rst", "adoc"];

#[derive(Debug, Deserialize, Validate)]
pub struct CreateJobRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 2000))]
    pub source_context: String,
    #[validate(length(min = 1, max = 2000))]
    pub target_context: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub id: String,
    pub status: String,
    pub name: String,
    pub source_context: String,
    pub target_context: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

impl From<crate::models::job::Job> for JobResponse {
    fn from(j: crate::models::job::Job) -> Self {
        Self {
            id: j.id.to_string(),
            status: j.status,
            name: j.name,
            source_context: j.source_context,
            target_context: j.target_context,
            created_at: j.created_at.to_rfc3339(),
            updated_at: j.updated_at.to_rfc3339(),
            completed_at: j.completed_at.map(|t| t.to_rfc3339()),
            error: j.error,
        }
    }
}

#[tracing::instrument(skip(state, claims))]
pub async fn create_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobResponse>), AppError> {
    body.validate()
        .map_err(|e| AppError::UnprocessableEntity(e.to_string()))?;

    let job = db::jobs::create(&state.db, claims.sub, &body.name, &body.source_context, &body.target_context).await?;
    metrics::counter!("docforge_jobs_total", "status" => "pending").increment(1);

    tracing::info!(job_id = %job.id, user_id = %claims.sub, "job created");

    Ok((StatusCode::CREATED, Json(job.into())))
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_jobs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<JobResponse>>, AppError> {
    let limit = q.limit.unwrap_or(20).min(100);
    let offset = q.offset.unwrap_or(0);

    let jobs = db::jobs::list_by_user(&state.db, claims.sub, limit, offset).await?;
    Ok(Json(jobs.into_iter().map(Into::into).collect()))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobResponse>, AppError> {
    let job = db::jobs::find_by_id(&state.db, id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))?;

    Ok(Json(job.into()))
}

#[tracing::instrument(skip(state, claims))]
pub async fn delete_job(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = db::jobs::delete(&state.db, id, claims.sub).await?;

    if !deleted {
        return Err(AppError::NotFound(format!("job {id} not found")));
    }

    tracing::info!(job_id = %id, user_id = %claims.sub, "job deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state, claims, multipart))]
pub async fn upload_documents(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    // Verify job belongs to user
    let _ = db::jobs::find_by_id(&state.db, id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))?;

    let max_bytes = state.config.max_file_size_bytes();
    let mut new_docs = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        let filename = field
            .file_name()
            .ok_or_else(|| AppError::BadRequest("file name required".into()))?
            .to_string();

        validate_filename(&filename)?;

        let data: Bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("failed to read file: {e}")))?;

        if data.len() > max_bytes {
            return Err(AppError::BadRequest(format!(
                "file {} exceeds max size of {}MB",
                filename, state.config.max_file_size_mb
            )));
        }

        let content = std::str::from_utf8(&data)
            .map_err(|_| AppError::BadRequest(format!("{filename} is not valid UTF-8")))?
            .to_string();

        new_docs.push(crate::db::documents::NewDocument {
            job_id: id,
            filename,
            content,
            doc_type: "input".to_string(),
        });

        if new_docs.len() > MAX_FILES_PER_JOB {
            return Err(AppError::BadRequest(format!(
                "max {MAX_FILES_PER_JOB} files per job"
            )));
        }
    }

    if new_docs.is_empty() {
        return Err(AppError::BadRequest("no files uploaded".into()));
    }

    db::documents::create_batch(&state.db, new_docs).await?;
    enqueue_job(&state.redis, id)
        .await
        .map_err(AppError::Internal)?;

    tracing::info!(job_id = %id, "documents uploaded, job enqueued");
    Ok(StatusCode::ACCEPTED)
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::document::Document>>, AppError> {
    let _ = db::jobs::find_by_id(&state.db, id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))?;

    let docs = db::documents::find_by_job(&state.db, id, "input").await?;
    Ok(Json(docs))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_result(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::document::Document>>, AppError> {
    let job = db::jobs::find_by_id(&state.db, id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))?;

    if job.status != "completed" {
        return Err(AppError::BadRequest(format!(
            "job is not completed (status: {})",
            job.status
        )));
    }

    let docs = db::documents::find_by_job(&state.db, id, "output").await?;
    Ok(Json(docs))
}

fn validate_filename(name: &str) -> Result<(), AppError> {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(AppError::BadRequest(format!(
            "file type .{ext} not allowed; allowed: {}",
            ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    Ok(())
}
