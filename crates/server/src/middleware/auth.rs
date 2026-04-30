use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use argon2::PasswordVerifier;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    db::entities::{service_keys, ServiceKeys},
    AppError, AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct ServiceKey {
    pub id: uuid::Uuid,
    pub name: String,
    pub scopes: serde_json::Value,
}

pub async fn admin_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;

    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    if token_data.claims.sub != "admin" {
        return Err(AppError::Unauthorized);
    }

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

pub async fn service_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;

    // Fast reject: must start with tks_
    if !token.starts_with("tks_") {
        return Err(AppError::Unauthorized);
    }

    // Extract prefix (first 8 chars of token for DB lookup)
    let prefix = if token.len() >= 8 {
        &token[0..8]
    } else {
        return Err(AppError::Unauthorized);
    };

    let key_record = ServiceKeys::find()
        .filter(service_keys::Column::TokenPrefix.eq(prefix))
        .filter(service_keys::Column::Enabled.eq(true))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .ok_or(AppError::Unauthorized)?;

    // Check expiration
    if let Some(expires_at) = key_record.expires_at {
        if chrono::Utc::now() > expires_at.naive_utc().and_utc() {
            return Err(AppError::Unauthorized);
        }
    }

    // Verify token hash using argon2
    argon2::Argon2::default()
        .verify_password(
            token.as_bytes(),
            &argon2::PasswordHash::new(&key_record.token_hash).map_err(|_| AppError::Unauthorized)?,
        )
        .map_err(|_| AppError::Unauthorized)?;

    let service_key = ServiceKey {
        id: key_record.id,
        name: key_record.name,
        scopes: key_record.scopes,
    };

    req.extensions_mut().insert(service_key);

    Ok(next.run(req).await)
}
