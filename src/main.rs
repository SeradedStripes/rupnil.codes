use axum::{routing::get, Router, http::StatusCode};
use std::net::SocketAddr;
use std::sync::Arc;
use base64::Engine;
use sqlx::Row;

mod routes;
mod auth;
mod services;
mod handlers;
mod models;
mod db;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let _ = dotenv::dotenv();

    
    
    
    let app_and_state: Option<(Router<std::sync::Arc<state::AppState>>, std::sync::Arc<state::AppState>)> = if let Ok(database_url) = std::env::var("DATABASE_URL") {
        tracing::info!("Using DATABASE_URL from environment");
        let pool = db::init_pool(&database_url).await?;

        let hca_client = services::hca_client::HcaClient::new(
            std::env::var("HCA_HOST").unwrap_or_else(|_| "https://auth.hackclub.com".to_string()),
            std::env::var("HCA_CLIENT_ID").expect("HCA_CLIENT_ID must be set"),
            std::env::var("HCA_CLIENT_SECRET").expect("HCA_CLIENT_SECRET must be set"),
            std::env::var("HCA_CALLBACK_URL").unwrap_or_else(|_| "http://localhost:8000/oauth/callback".to_string()),
        );

        let master_key_env = std::env::var("MASTER_KEY").expect("MASTER_KEY must be set");
        let master_key = services::crypto::parse_master_key(&master_key_env)?;

        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let state = state::AppState::new(
            pool,
            hca_client,
            master_key,
            jwt_secret,
            std::env::var("REFRESH_TOKEN_EXPIRY_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(60 * 60 * 24 * 30),
        );


        let shared_state = Arc::new(state);

        let router = Router::new()
            .route("/health", get(|| async { (StatusCode::OK, "OK") }))
            .merge(routes::router())
            .with_state(shared_state.clone());

        Some((router, shared_state))
    } else {
        tracing::warn!("DATABASE_URL not set — running in minimal static dev mode");
        None
    };

    
    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!(%addr, "Listening on http://{}", addr);

    
    
    
    
    tracing::info!("Starting server at http://{}", addr);

    
    
    
    use hyper::{Body, Request, Response, StatusCode, Server};
    use hyper::service::{make_service_fn, service_fn};
    use std::convert::Infallible;

    let make_svc = make_service_fn(move |_| {
        
        let state_clone = app_and_state.as_ref().map(|(_, s)| s.clone());
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let state_clone = state_clone.clone();
                async move {
                    use hyper::Method;
                    use serde::Deserialize;

                    
                    if req.method() == Method::OPTIONS {
                        let mut resp = Response::new(Body::empty());
                        let headers = resp.headers_mut();
                        headers.insert("access-control-allow-origin", "*".parse().unwrap());
                        headers.insert("access-control-allow-methods", "GET, POST, OPTIONS".parse().unwrap());
                        headers.insert("access-control-allow-headers", "Authorization, Content-Type".parse().unwrap());
                        return Ok::<_, Infallible>(resp);
                    }

                    let uri_path = req.uri().path().to_string();
                    let method = req.method().clone();

                    
                    if uri_path == "/health" && method == Method::GET {
                        let mut resp = Response::new(Body::from("OK"));
                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                        return Ok::<_, Infallible>(resp);
                    }

                    
                    let s = match &state_clone {
                        Some(s) => s.clone(),
                        None => {
                            let mut resp = Response::new(Body::from("DB not configured"));
                            *resp.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                            return Ok::<_, Infallible>(resp);
                        }
                    };

                    
                    if uri_path == "/auth/hack_club" && method == Method::GET {
                        let state_token = base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8; 16]>());
                        let url = s.hca.auth_url(&state_token);
                        let mut resp = Response::new(Body::empty());
                        *resp.status_mut() = StatusCode::FOUND;
                        resp.headers_mut().insert(hyper::header::LOCATION, url.parse().unwrap());
                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                        return Ok::<_, Infallible>(resp);
                    }

                    
                    if uri_path == "/oauth/callback" && method == Method::GET {
                        if let Some(q) = req.uri().query() {
                            let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(q.as_bytes()).into_owned().collect();
                            if let Some(code) = params.get("code") {
                                match crate::handlers::auth::oauth_exchange_and_upsert(&*s, code).await {
                                    Ok((jwt, rt)) => {
                                        
                                        
                                        
                                        let payload_json = serde_json::to_string(&serde_json::json!({"jwt": jwt, "refresh_token": rt})).unwrap();
                                        let html = format!(r#"<!doctype html>
<html><head><meta charset=\"utf-8\"></head><body>
<script>
(function(){{
  var payload = {payload};
  try {{
    if (window.opener && window.opener !== window) {{
      window.opener.postMessage(payload, "*");
      window.close();
    }} else {{
      document.body.innerText = JSON.stringify(payload);
    }}
  }} catch (e) {{ document.body.innerText = JSON.stringify(payload); }}
}})();
</script>
</body></html>"#, payload=payload_json);
                                        let mut resp = Response::new(Body::from(html));
                                        resp.headers_mut().insert(hyper::header::CONTENT_TYPE, "text/html".parse().unwrap());
                                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                        return Ok::<_, Infallible>(resp);
                                    }
                                    Err(e) => {
                                        let mut resp = Response::new(Body::from(format!("OAuth error: {}", e)));
                                        *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                        return Ok::<_, Infallible>(resp);
                                    }
                                }
                            }
                        }
                        let mut resp = Response::new(Body::from("Missing code"));
                        *resp.status_mut() = StatusCode::BAD_REQUEST;
                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                        return Ok::<_, Infallible>(resp);
                    }

                    
                    if uri_path == "/auth/refresh" && method == Method::POST {
                        #[derive(Deserialize)]
                        struct RefreshReq { refresh_token: String }

                        let whole = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
                        let parsed: Result<RefreshReq, _> = serde_json::from_slice(&whole);
                        if parsed.is_err() {
                            let mut resp = Response::new(Body::from("Bad JSON"));
                            *resp.status_mut() = StatusCode::BAD_REQUEST;
                            return Ok::<_, Infallible>(resp);
                        }
                        let parsed = parsed.unwrap();

                        let rt_hash = crate::auth::hash_token(&parsed.refresh_token);
                        let row = sqlx::query("SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND expires_at > now()")
                            .bind(&rt_hash)
                            .fetch_optional(&*s.pool)
                            .await;

                        match row {
                            Ok(Some(r)) => {
                                let user_id: uuid::Uuid = r.try_get("user_id").unwrap();
                                let _ = sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1").bind(&rt_hash).execute(&*s.pool).await;

                                let new_rt_bytes: [u8; 32] = rand::random();
                                let new_rt = base64::engine::general_purpose::STANDARD.encode(&new_rt_bytes);
                                let new_rt_hash = crate::auth::hash_token(&new_rt);
                                let expires_at = chrono::Utc::now() + chrono::Duration::seconds(s.refresh_token_expiry_seconds);
                                let _ = sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
                                    .bind(user_id)
                                    .bind(new_rt_hash)
                                    .bind(expires_at)
                                    .execute(&*s.pool)
                                    .await;

                                let jwt = crate::auth::create_jwt(&user_id.to_string(), &s.jwt_secret, 3600);
                                match jwt {
                                    Ok(j) => {
                                        let body = format!("{{\"jwt\":\"{}\",\"refresh_token\":\"{}\"}}", j, new_rt);
                                        let mut resp = Response::new(Body::from(body));
                                        resp.headers_mut().insert(hyper::header::CONTENT_TYPE, "application/json".parse().unwrap());
                                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                        return Ok::<_, Infallible>(resp);
                                    }
                                    Err(e) => {
                                        let mut resp = Response::new(Body::from(format!("JWT creation error: {}", e)));
                                        *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                        return Ok::<_, Infallible>(resp);
                                    }
                                }
                            }
                            Ok(None) => {
                                let mut resp = Response::new(Body::from("Invalid or expired refresh token"));
                                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                            Err(e) => {
                                let mut resp = Response::new(Body::from(format!("DB error: {}", e)));
                                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                        }
                    }

                    
                    if uri_path == "/auth/logout" && method == Method::POST {
                        #[derive(Deserialize)]
                        struct LogoutReq { refresh_token: String }

                        let whole = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
                        let parsed: Result<LogoutReq, _> = serde_json::from_slice(&whole);
                        if parsed.is_err() {
                            let mut resp = Response::new(Body::from("Bad JSON"));
                            *resp.status_mut() = StatusCode::BAD_REQUEST;
                            return Ok::<_, Infallible>(resp);
                        }
                        let parsed = parsed.unwrap();
                        let rt_hash = crate::auth::hash_token(&parsed.refresh_token);
                        let _ = sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1").bind(&rt_hash).execute(&*s.pool).await;
                        let mut resp = Response::new(Body::empty());
                        *resp.status_mut() = StatusCode::NO_CONTENT;
                        resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                        return Ok::<_, Infallible>(resp);
                    }

                    
                    if uri_path == "/me" && method == Method::GET {
                        let headers = req.headers();
                        let auth = headers.get("authorization");
                        if auth.is_none() {
                            let mut resp = Response::new(Body::from("Missing Authorization header"));
                            *resp.status_mut() = StatusCode::UNAUTHORIZED;
                            resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                            return Ok::<_, Infallible>(resp);
                        }
                        let auth = auth.unwrap().to_str().unwrap_or("");
                        if !auth.starts_with("Bearer ") {
                            let mut resp = Response::new(Body::from("Malformed Authorization header"));
                            *resp.status_mut() = StatusCode::UNAUTHORIZED;
                            resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                            return Ok::<_, Infallible>(resp);
                        }
                        let token = auth.trim_start_matches("Bearer ").trim();
                        let user_id_str = match crate::auth::verify_jwt(token, &s.jwt_secret) {
                            Ok(u) => u,
                            Err(_) => {
                                let mut resp = Response::new(Body::from("Invalid token"));
                                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                        };
                        let user_id = match uuid::Uuid::parse_str(&user_id_str) {
                            Ok(id) => id,
                            Err(_) => {
                                let mut resp = Response::new(Body::from("Invalid token subject"));
                                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                        };

                        let user_row = sqlx::query_as::<_, crate::models::user::User>("SELECT id, email, display_name FROM users WHERE id = $1")
                            .bind(user_id)
                            .fetch_optional(&*s.pool)
                            .await;

                        match user_row {
                            Ok(Some(user)) => {
                                let body = serde_json::to_string(&user).unwrap_or_else(|_| "{}".to_string());
                                let mut resp = Response::new(Body::from(body));
                                resp.headers_mut().insert(hyper::header::CONTENT_TYPE, "application/json".parse().unwrap());
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                            Ok(None) => {
                                let mut resp = Response::new(Body::from("User not found"));
                                *resp.status_mut() = StatusCode::NOT_FOUND;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                            Err(e) => {
                                let mut resp = Response::new(Body::from(format!("DB error: {}", e)));
                                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                                return Ok::<_, Infallible>(resp);
                            }
                        }
                    }

                    
                    let mut resp = Response::new(Body::from("Not found"));
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
                    Ok::<_, Infallible>(resp)
                }
            }))
        }
    });

    Server::bind(&addr).serve(make_svc).await?;

    Ok(())
}
