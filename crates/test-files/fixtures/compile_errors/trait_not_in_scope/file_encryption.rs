
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::FromHex;
use rand::RngCore;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read input file: {}", e))?;

    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&ciphertext).map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<(), String> {
    let mut file = File::open(input_path).map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let decrypted = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output.write_all(&decrypted).map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn generate_random_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

pub fn key_from_hex(hex_str: &str) -> Result<[u8; 32], String> {
    let bytes = Vec::from_hex(hex_str).map_err(|e| format!("Invalid hex string: {}", e))?;
    if bytes.len() != 32 {
        return Err("Key must be 32 bytes (64 hex characters)".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

pub fn iv_from_hex(hex_str: &str) -> Result<[u8; 16], String> {
    let bytes = Vec::from_hex(hex_str).map_err(|e| format!("Invalid hex string: {}", e))?;
    if bytes.len() != 16 {
        return Err("IV must be 16 bytes (32 hex characters)".to_string());
    }
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&bytes);
    Ok(iv)
}