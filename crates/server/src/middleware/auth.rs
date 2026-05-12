use argon2::PasswordVerifier;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
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

/// Validate a raw bearer token against the `service_keys` table.
///
/// Returns `Some(ServiceKey)` when the token is well-formed, exists,
/// is enabled, not expired, and its argon2 hash verifies. Returns
/// `None` for every failure mode (malformed / unknown / disabled /
/// expired / hash mismatch / DB error). Callers decide how to react —
/// the [`service_auth`] middleware turns `None` into `401`, while
/// `/api/capabilities` degrades to a public view.
pub async fn validate_service_key(state: &AppState, raw_token: &str) -> Option<ServiceKey> {
    if !raw_token.starts_with("tks_") || raw_token.len() < 8 {
        return None;
    }
    let prefix = &raw_token[0..8];

    let key_record = ServiceKeys::find()
        .filter(service_keys::Column::TokenPrefix.eq(prefix))
        .filter(service_keys::Column::Enabled.eq(true))
        .one(&state.db)
        .await
        .ok()??;

    if let Some(expires_at) = key_record.expires_at {
        if chrono::Utc::now() > expires_at.naive_utc().and_utc() {
            return None;
        }
    }

    let parsed_hash = argon2::PasswordHash::new(&key_record.token_hash).ok()?;
    argon2::Argon2::default()
        .verify_password(raw_token.as_bytes(), &parsed_hash)
        .ok()?;

    Some(ServiceKey {
        id: key_record.id,
        name: key_record.name,
        scopes: key_record.scopes,
    })
}

/// Extract the bearer token from an `Authorization` header value.
pub fn bearer_token(header_value: &str) -> Option<&str> {
    header_value.strip_prefix("Bearer ")
}

pub async fn service_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(bearer_token)
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let service_key = validate_service_key(&state, &token)
        .await
        .ok_or(AppError::Unauthorized)?;

    req.extensions_mut().insert(service_key);

    Ok(next.run(req).await)
}
