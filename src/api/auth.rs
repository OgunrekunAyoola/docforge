use crate::api::middleware::auth::Claims;
use crate::{api::AppState, db, error::AppError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
}

#[tracing::instrument(skip(state, body), fields(email = %body.email))]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::UnprocessableEntity(e.to_string()))?;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        .to_string();

    let user = db::users::create(&state.db, &body.email, &hash).await?;

    tracing::info!(user_id = %user.id, "user registered");

    let token = mint_token(
        &user.id,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        email: user.email,
    }))
}

#[tracing::instrument(skip(state, body), fields(email = %body.email))]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::UnprocessableEntity(e.to_string()))?;

    let user = db::users::find_by_email(&state.db, &body.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))?;

    tracing::info!(user_id = %user.id, "user logged in");

    let token = mint_token(
        &user.id,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        email: user.email,
    }))
}

fn mint_token(
    user_id: &uuid::Uuid,
    email: &str,
    secret: &str,
    expiry_hours: u64,
) -> Result<String, AppError> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expiry_hours as i64))
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("timestamp overflow")))?
        .timestamp() as usize;

    let claims = Claims {
        sub: *user_id,
        email: email.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))
}
