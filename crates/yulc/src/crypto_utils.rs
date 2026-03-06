use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::RngCore;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

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

pub fn encrypt_aes_256_cbc(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> String {
    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    hex::encode(ciphertext)
}

pub fn decrypt_aes_256_cbc(ciphertext_hex: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>, String> {
    let ciphertext = hex::decode(ciphertext_hex).map_err(|e| e.to_string())?;
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| e.to_string())?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_random_key();
        let iv = generate_random_iv();
        let original_message = b"Secret message for AES-256-CBC encryption test.";

        let encrypted = encrypt_aes_256_cbc(original_message, &key, &iv);
        let decrypted = decrypt_aes_256_cbc(&encrypted, &key, &iv).unwrap();

        assert_eq!(original_message.to_vec(), decrypted);
    }

    #[test]
    fn test_invalid_ciphertext() {
        let key = generate_random_key();
        let iv = generate_random_iv();
        let invalid_hex = "not_a_valid_hex_string";

        let result = decrypt_aes_256_cbc(invalid_hex, &key, &iv);
        assert!(result.is_err());
    }
}