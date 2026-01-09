use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};

const NONCE_SIZE: usize = 12;

pub fn encrypt_aes256_gcm(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_SIZE]>());

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .context("Encryption failed")?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_aes256_gcm(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE {
        anyhow::bail!("Ciphertext too short");
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted_data)
        .context("Decryption failed")
}