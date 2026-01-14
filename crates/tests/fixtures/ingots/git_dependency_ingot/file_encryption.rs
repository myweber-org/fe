use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::{rngs::OsRng, RngCore};
use sha2::Sha256;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub fn encrypt_file_data(data: &[u8], password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params::default(),
    )?;

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(data);

    let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + ciphertext.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&iv);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_file_data(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() < SALT_LEN + IV_LEN {
        return Err("Invalid encrypted data length".into());
    }

    let salt = &encrypted_data[0..SALT_LEN];
    let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
    let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
        Params::default(),
    )?;

    let cipher = Aes256CbcDec::new(&key.into(), iv.into());
    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original_data = b"Secret document content";
        let password = "strong_password_123";

        let encrypted = encrypt_file_data(original_data, password).unwrap();
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len());

        let decrypted = decrypt_file_data(&encrypted, password).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Test data";
        let encrypted = encrypt_file_data(data, "correct_pass").unwrap();
        let result = decrypt_file_data(&encrypted, "wrong_pass");
        assert!(result.is_err());
    }
}