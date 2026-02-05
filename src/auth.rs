use anyhow::Result;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(user_id: &str, secret: &str, expiry_seconds: i64) -> Result<String> {
    let exp = Utc::now().checked_add_signed(Duration::seconds(expiry_seconds)).unwrap().timestamp();
    let claims = Claims { sub: user_id.to_string(), exp: exp as usize };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;
    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<String> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())?;
    Ok(data.claims.sub)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}
