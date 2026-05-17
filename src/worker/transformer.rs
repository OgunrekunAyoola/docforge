use crate::{
    db,
    db::documents::NewDocument,
    models::document::Document,
    ollama::{prompt::build_prompt, OllamaClient},
};
use anyhow::{Context, Result};
use futures::future::join_all;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(skip(pool, ollama), fields(%job_id))]
pub async fn process_job(pool: &PgPool, ollama: &Arc<OllamaClient>, job_id: Uuid) -> Result<()> {
    db::jobs::update_status(pool, job_id, "processing", None).await?;
    metrics::counter!("docforge_jobs_total", "status" => "processing").increment(1);

    let job = db::jobs::find_by_id_any_user(pool, job_id)
        .await?
        .with_context(|| format!("job {job_id} not found"))?;

    let input_docs = db::documents::find_by_job(pool, job_id, "input").await?;

    if input_docs.is_empty() {
        tracing::warn!(%job_id, "no input documents, marking complete");
        db::jobs::update_status(pool, job_id, "completed", None).await?;
        return Ok(());
    }

    let tasks: Vec<_> = input_docs
        .into_iter()
        .map(|doc| {
            let ollama = Arc::clone(ollama);
            let source = job.source_context.clone();
            let target = job.target_context.clone();
            async move { transform_document(&ollama, &source, &target, doc).await }
        })
        .collect();

    let timer = std::time::Instant::now();
    let results = join_all(tasks).await;
    let elapsed = timer.elapsed().as_secs_f64();

    metrics::histogram!("docforge_job_duration_seconds").record(elapsed);

    let mut output_docs = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(doc) => output_docs.push(doc),
            Err(e) => errors.push(e.to_string()),
        }
    }

    let doc_count = output_docs.len() as u64;
    if !output_docs.is_empty() {
        db::documents::create_batch(pool, output_docs).await?;
        metrics::counter!("docforge_documents_processed_total").increment(doc_count);
    }

    if errors.is_empty() {
        db::jobs::update_status(pool, job_id, "completed", None).await?;
        metrics::counter!("docforge_jobs_total", "status" => "completed").increment(1);
        tracing::info!(%job_id, "job completed");
    } else {
        let error_msg = errors.join("; ");
        db::jobs::update_status(pool, job_id, "failed", Some(&error_msg)).await?;
        metrics::counter!("docforge_jobs_total", "status" => "failed").increment(1);
        tracing::error!(%job_id, %error_msg, "job failed");
        return Err(anyhow::anyhow!(error_msg));
    }

    Ok(())
}

#[tracing::instrument(skip(ollama, doc), fields(filename = %doc.filename))]
async fn transform_document(
    ollama: &OllamaClient,
    source_context: &str,
    target_context: &str,
    doc: Document,
) -> Result<NewDocument> {
    let sanitized = sanitize_content(&doc.content);
    let prompt = build_prompt(source_context, target_context, &doc.filename, &sanitized);

    let start = std::time::Instant::now();
    let transformed = ollama.generate(&prompt).await?;
    let elapsed = start.elapsed().as_secs_f64();

    metrics::histogram!("docforge_ollama_request_duration_seconds").record(elapsed);

    Ok(NewDocument {
        job_id: doc.job_id,
        filename: doc.filename,
        content: transformed,
        doc_type: "output".to_string(),
    })
}

fn sanitize_content(content: &str) -> String {
    // Strip null bytes and enforce valid UTF-8 replacement
    content
        .chars()
        .filter(|c| *c != '\0')
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_null_bytes() {
        let input = "hello\0world";
        assert_eq!(sanitize_content(input), "helloworld");
    }

    #[test]
    fn sanitize_preserves_newlines_and_tabs() {
        let input = "line1\nline2\ttabbed\r\n";
        let out = sanitize_content(input);
        assert!(out.contains('\n'));
        assert!(out.contains('\t'));
    }

    #[test]
    fn sanitize_strips_control_chars() {
        let input = "clean\x01\x02\x1btext";
        let out = sanitize_content(input);
        assert_eq!(out, "cleantext");
    }

    #[test]
    fn sanitize_passthrough_normal_content() {
        let input = "# Heading\n\nSome **markdown** content.";
        assert_eq!(sanitize_content(input), input);
    }
}
