
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct FileEncryptor {
    password: String,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
        }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let plaintext = fs::read(input_path).map_err(|e| e.to_string())?;
        
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        
        let key = self.derive_key(&salt);
        
        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);
        
        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let data = fs::read(input_path).map_err(|e| e.to_string())?;
        
        if data.len() < SALT_LEN + IV_LEN {
            return Err("File too short".to_string());
        }
        
        let salt = &data[..SALT_LEN];
        let iv = &data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &data[SALT_LEN + IV_LEN..];
        
        let key = self.derive_key(salt);
        
        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| e.to_string())?;
        
        fs::write(output_path, &plaintext).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn derive_key(&self, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            self.password.as_bytes(),
            salt,
            KEY_ITERATIONS,
            &mut key,
        );
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_roundtrip() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password);
        
        let original_data = b"Hello, this is a secret message!";
        let plaintext_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(plaintext_file.path(), original_data).unwrap();
        
        encryptor.encrypt_file(
            plaintext_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}