use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            Self::UnprocessableEntity(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR"),
            Self::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED"),
            Self::Internal(_) | Self::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Internal(_) | Self::Database(_) => "An internal error occurred".to_string(),
            other => other.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "internal server error");
        }

        let body = json!({
            "error": {
                "code": code,
                "message": self.message()
            }
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_404() {
        let err = AppError::NotFound("job abc".into());
        let (status, code) = err.status_and_code();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(code, "NOT_FOUND");
    }

    #[test]
    fn unauthorized_maps_to_401() {
        let err = AppError::Unauthorized("bad token".into());
        let (status, _) = err.status_and_code();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn conflict_maps_to_409() {
        let err = AppError::Conflict("email taken".into());
        let (status, _) = err.status_and_code();
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn rate_limit_maps_to_429() {
        let err = AppError::TooManyRequests;
        let (status, code) = err.status_and_code();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(code, "RATE_LIMIT_EXCEEDED");
    }

    #[test]
    fn internal_error_hides_message() {
        let err = AppError::Internal(anyhow::anyhow!("secret db credentials leaked"));
        assert_eq!(err.message(), "An internal error occurred");
    }
}
