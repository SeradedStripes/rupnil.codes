use crate::services::hca_client::HcaClient;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub hca: Arc<HcaClient>,
    pub master_key: Arc<Vec<u8>>,
    pub jwt_secret: Arc<String>,
    pub refresh_token_expiry_seconds: i64,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        hca: HcaClient,
        master_key: Vec<u8>,
        jwt_secret: String,
        refresh_token_expiry_seconds: i64,
    ) -> Self {
        Self {
            pool: Arc::new(pool),
            hca: Arc::new(hca),
            master_key: Arc::new(master_key),
            jwt_secret: Arc::new(jwt_secret),
            refresh_token_expiry_seconds,
        }
    }
}
