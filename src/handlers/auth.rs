use axum::{extract::{Query, Json, State}, response::IntoResponse, http::StatusCode};
use serde::Deserialize;
use crate::state::AppState;
use sqlx::{Row};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
use anyhow::Result;

#[derive(Deserialize)]
pub struct OAuthQuery {
    code: String,
    state: Option<String>,
}

pub async fn start_hca_auth(State(state): State<std::sync::Arc<AppState>>) -> impl IntoResponse {
    
    let state_token = base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8; 16]>());
    let url = state.hca.auth_url(&state_token);
    axum::response::Redirect::temporary(&url)
}

pub async fn oauth_callback(State(state): State<std::sync::Arc<AppState>>, Query(query): Query<OAuthQuery>) -> impl IntoResponse {
    
    match oauth_exchange_and_upsert(&state, &query.code).await {
        Ok((jwt, refresh_token)) => (
            StatusCode::OK,
            format!("{{\"jwt\":\"{}\",\"refresh_token\":\"{}\"}}", jwt, refresh_token),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth error: {}", e)),
    }
}

pub async fn oauth_exchange_and_upsert(state: &AppState, code: &str) -> Result<(String, String)> {
    let (access_token, refresh_token_opt) = state.hca.exchange_code(code).await?;
    let me = state.hca.fetch_me(&access_token).await?;

    
    let slack_id = me.slack_id.ok_or_else(|| anyhow::anyhow!("missing slack_id in HCA profile"))?;
    let email = me.email.ok_or_else(|| anyhow::anyhow!("missing email in HCA profile"))?;

    

    
    let user_row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&*state.pool)
        .await?;

    let user_id: uuid::Uuid = if let Some(row) = user_row {
        row.try_get("id")?
    } else {
        let rec = sqlx::query("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(&email)
            .bind(&me.display_name)
            .fetch_one(&*state.pool)
            .await?;
        rec.try_get("id")?
    };

    
    let identity_row = sqlx::query("SELECT id FROM user_identities WHERE provider = $1 AND uid = $2")
        .bind("hack_club")
        .bind(&me.id)
        .fetch_optional(&*state.pool)
        .await?;

    let identity_id: uuid::Uuid = if let Some(row) = identity_row {
        row.try_get("id")?
    } else {
        let rec = sqlx::query("INSERT INTO user_identities (user_id, provider, uid, slack_id) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(user_id)
            .bind("hack_club")
            .bind(&me.id)
            .bind(&slack_id)
            .fetch_one(&*state.pool)
            .await?;
        rec.try_get("id")?
    };

    
    let master_key = state.master_key.as_slice();
    let (enc_access, nonce) = crate::services::crypto::encrypt(master_key, access_token.as_bytes())?;
    let (enc_refresh, nonce_refresh) = if let Some(rt) = refresh_token_opt.as_deref() {
        let (c, n) = crate::services::crypto::encrypt(master_key, rt.as_bytes())?;
        (Some(c), Some(n))
    } else {
        (None, None)
    };

    sqlx::query("INSERT INTO user_tokens (identity_id, encrypted_access_token, encrypted_refresh_token, nonce, nonce_refresh) VALUES ($1, $2, $3, $4, $5)")
        .bind(identity_id)
        .bind(enc_access)
        .bind(enc_refresh)
        .bind(nonce)
        .bind(nonce_refresh)
        .execute(&*state.pool)
        .await?;

    
    let rt_bytes: [u8; 32] = rand::random();
    let rt = general_purpose::STANDARD.encode(&rt_bytes);
    let mut hasher = Sha256::new();
    hasher.update(rt.as_bytes());
    let rt_hash = general_purpose::STANDARD.encode(hasher.finalize());
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(state.refresh_token_expiry_seconds);

    sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(rt_hash)
        .bind(expires_at)
        .execute(&*state.pool)
        .await?;


    
    let jwt = crate::auth::create_jwt(&user_id.to_string(), &state.jwt_secret, 3600)?;

    Ok((jwt, rt))
}

#[derive(Deserialize)]
pub struct MagicRequest {
    email: String,
}

pub async fn request_magic_link(Json(payload): Json<MagicRequest>) -> impl IntoResponse {
    
    (StatusCode::ACCEPTED, format!("If eligible, a magic link will be sent to {}", payload.email))
}

pub async fn consume_magic_link(Query(params): Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    
    if let Some(token) = params.get("token") {
        
        (StatusCode::OK, format!("Consumed token: {}", token))
    } else {
        (StatusCode::BAD_REQUEST, "missing token".to_string())
    }
}

use axum::http::HeaderMap;

#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(State(state): State<std::sync::Arc<AppState>>, Json(payload): Json<RefreshRequest>) -> impl IntoResponse {
    
    let rt_hash = crate::auth::hash_token(&payload.refresh_token);
    let row = sqlx::query("SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND expires_at > now()")
        .bind(&rt_hash)
        .fetch_optional(&*state.pool)
        .await;

    match row {
        Ok(Some(r)) => {
            let user_id: uuid::Uuid = r.try_get("user_id").unwrap();
            
            let _ = sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1").bind(&rt_hash).execute(&*state.pool).await;
            
            let new_rt_bytes: [u8; 32] = rand::random();
            let new_rt = base64::engine::general_purpose::STANDARD.encode(&new_rt_bytes);
            let new_rt_hash = crate::auth::hash_token(&new_rt);
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(state.refresh_token_expiry_seconds);
            let _ = sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
.bind(user_id)
            .bind(new_rt_hash)
            .bind(expires_at)
                .execute(&*state.pool)
                .await;

            let jwt = crate::auth::create_jwt(&user_id.to_string(), &state.jwt_secret, 3600);
            match jwt {
                Ok(j) => (StatusCode::OK, format!("{{\"jwt\":\"{}\",\"refresh_token\":\"{}\"}}", j, new_rt)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT creation error: {}", e)),
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Invalid or expired refresh token".to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)),
    }
}

#[derive(serde::Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub async fn logout(State(state): State<std::sync::Arc<AppState>>, Json(payload): Json<LogoutRequest>) -> impl IntoResponse {
    let rt_hash = crate::auth::hash_token(&payload.refresh_token);
    let _res = sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1").bind(&rt_hash).execute(&*state.pool).await;
    (StatusCode::NO_CONTENT, "".to_string())
}

pub async fn me(State(state): State<std::sync::Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    
    let auth = headers.get("authorization");
    if auth.is_none() {
        return (StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string());
    }
    let auth = auth.unwrap().to_str().unwrap_or("");
    if !auth.starts_with("Bearer ") {
        return (StatusCode::UNAUTHORIZED, "Malformed Authorization header".to_string());
    }
    let token = auth.trim_start_matches("Bearer ").trim();
    let user_id_str = match crate::auth::verify_jwt(token, &state.jwt_secret) {
        Ok(u) => u,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
    };
    let user_id = match uuid::Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid token subject".to_string()),
    };

    let user_row = sqlx::query_as::<_, crate::models::user::User>("SELECT id, email, display_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&*state.pool)
        .await;

    match user_row {
        Ok(Some(user)) => (StatusCode::OK, serde_json::to_string(&user).unwrap()),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found".to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)),
    }
}
