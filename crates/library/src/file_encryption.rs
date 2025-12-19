
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex::{decode, encode};
use rand::RngCore;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub struct EncryptionResult {
    pub ciphertext: String,
    pub iv: String,
}

pub fn encrypt_data(plaintext: &str, key: &[u8; 32]) -> Result<EncryptionResult, Box<dyn Error>> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes for AES-256".into());
    }

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new(key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    Ok(EncryptionResult {
        ciphertext: encode(&ciphertext),
        iv: encode(iv),
    })
}

pub fn decrypt_data(ciphertext: &str, key: &[u8; 32], iv: &str) -> Result<String, Box<dyn Error>> {
    let ciphertext_bytes = decode(ciphertext)?;
    let iv_bytes = decode(iv)?;

    if iv_bytes.len() != 16 {
        return Err("IV must be 16 bytes".into());
    }

    let decrypted = Aes256CbcDec::new(key.into(), &iv_bytes.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext_bytes)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = "Sensitive data requiring protection";

        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted.ciphertext, &key, &encrypted.iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0x42u8; 16];
        let plaintext = "Test data";

        let result = encrypt_data(plaintext, &short_key);
        assert!(result.is_err());
    }
}