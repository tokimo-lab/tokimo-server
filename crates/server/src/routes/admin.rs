use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use argon2::PasswordHasher;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use sea_orm::{entity::*, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::{
    db::entities::{service_keys, ServiceKeys},
    middleware::AdminClaims,
    AppError, AppResult, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/service-keys", get(list_keys).post(create_key).delete(delete_key))
        .route("/provider-configs", get(list_provider_configs))
        .route("/cache", get(list_cache))
}

#[derive(Deserialize)]
struct LoginRequest {
    bootstrap_key: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> AppResult<Json<LoginResponse>> {
    if req.bootstrap_key != state.config.admin_bootstrap_key {
        return Err(AppError::Unauthorized);
    }

    let claims = AdminClaims {
        sub: "admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Serialize)]
struct ServiceKeyResponse {
    id: uuid::Uuid,
    name: String,
    token_prefix: String,
    scopes: serde_json::Value,
    enabled: bool,
    expires_at: Option<String>,
    created_at: String,
    token: Option<String>,
}

async fn list_keys(State(state): State<AppState>) -> AppResult<Json<Vec<ServiceKeyResponse>>> {
    let keys = ServiceKeys::find()
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let response = keys
        .into_iter()
        .map(|k| ServiceKeyResponse {
            id: k.id,
            name: k.name,
            token_prefix: k.token_prefix,
            scopes: k.scopes,
            enabled: k.enabled,
            expires_at: k.expires_at.map(|dt| dt.to_string()),
            created_at: k.created_at.to_string(),
            token: None,
        })
        .collect();

    Ok(Json(response))
}

#[derive(Deserialize)]
struct CreateKeyRequest {
    name: String,
    scopes: Option<serde_json::Value>,
    expires_at: Option<String>,
}

async fn create_key(
    State(state): State<AppState>,
    Json(req): Json<CreateKeyRequest>,
) -> AppResult<Json<ServiceKeyResponse>> {
    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let token = format!("tks_{}", random_part);
    let token_prefix = token[0..8].to_string();

    let hash = argon2::Argon2::default()
        .hash_password(
            token.as_bytes(),
            &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to hash token: {}", e)))?
        .to_string();

    let expires_at = if let Some(exp_str) = req.expires_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&exp_str)
                .map_err(|e| AppError::BadRequest(format!("Invalid expires_at: {}", e)))?,
        )
    } else {
        None
    };

    let model = service_keys::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(req.name.clone()),
        token_hash: Set(hash),
        token_prefix: Set(token_prefix.clone()),
        scopes: Set(req.scopes.unwrap_or_else(|| serde_json::json!([]))),
        enabled: Set(true),
        expires_at: Set(expires_at),
        created_at: Set(chrono::Utc::now().into()),
    };

    let result = ServiceKeys::insert(model)
        .exec_with_returning(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ServiceKeyResponse {
        id: result.id,
        name: result.name,
        token_prefix: result.token_prefix,
        scopes: result.scopes,
        enabled: result.enabled,
        expires_at: result.expires_at.map(|dt| dt.to_string()),
        created_at: result.created_at.to_string(),
        token: Some(token),
    }))
}

#[derive(Deserialize)]
struct DeleteKeyRequest {
    id: uuid::Uuid,
}

async fn delete_key(
    State(state): State<AppState>,
    Json(req): Json<DeleteKeyRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ServiceKeys::delete_by_id(req.id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn list_provider_configs(State(_state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "configs": [] })))
}

async fn list_cache(State(_state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "entries": [] })))
}
