use axum::Router;

use crate::handlers;
use axum::routing::{get, post};

pub fn router() -> Router<std::sync::Arc<crate::state::AppState>> {
    Router::new()
        .route("/auth/hack_club", get(handlers::auth::start_hca_auth))
        .route("/oauth/callback", get(handlers::auth::oauth_callback))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/me", get(handlers::auth::me))
}

