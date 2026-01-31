
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::FromHex;
use rand::RngCore;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn generate_iv() -> [u8; 16] {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

pub fn encrypt_aes256_cbc(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }
    if iv.len() != 16 {
        return Err("IV must be 16 bytes".to_string());
    }

    let mut buffer = vec![0u8; plaintext.len() + 16];
    let len = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_b2b_mut::<Pkcs7>(plaintext, &mut buffer)
        .map_err(|e| e.to_string())?
        .len();

    buffer.truncate(len);
    Ok(buffer)
}

pub fn decrypt_aes256_cbc(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".to_string());
    }
    if iv.len() != 16 {
        return Err("IV must be 16 bytes".to_string());
    }

    let mut buffer = vec![0u8; ciphertext.len()];
    let len = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_b2b_mut::<Pkcs7>(ciphertext, &mut buffer)
        .map_err(|e| e.to_string())?
        .len();

    buffer.truncate(len);
    Ok(buffer)
}

pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    Vec::from_hex(hex_str).map_err(|e| e.to_string())
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}