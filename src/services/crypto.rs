use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use anyhow::Result;

pub fn parse_master_key(encoded: &str) -> Result<Vec<u8>> {
    
    let s = encoded.strip_prefix("base64:").unwrap_or(encoded);
    let bytes = general_purpose::STANDARD.decode(s)?;
    Ok(bytes)
}

pub fn encrypt(master_key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| anyhow::anyhow!(e))?;
    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt(master_key: &[u8], ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| anyhow::anyhow!(e))?;
    Ok(plaintext)
}
