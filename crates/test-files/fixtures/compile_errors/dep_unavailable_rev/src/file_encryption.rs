
use std::fs;
use std::io::{Read, Write};
use std::env;

fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_file> <output_file> <key>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let key = args[3].as_bytes();

    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let processed_data = xor_encrypt_decrypt(&buffer, key);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&processed_data)?;

    println!("Operation completed successfully.");
    Ok(())
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub enum CipherAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub struct FileEncryptor {
    algorithm: CipherAlgorithm,
}

impl FileEncryptor {
    pub fn new(algorithm: CipherAlgorithm) -> Self {
        FileEncryptor { algorithm }
    }

    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let data = self.read_file(input_path)?;
        let (ciphertext, key, nonce) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => self.aes_encrypt(&data)?,
            CipherAlgorithm::ChaCha20Poly1305 => self.chacha_encrypt(&data)?,
        };

        let mut output_data = Vec::new();
        output_data.extend_from_slice(key.as_slice());
        output_data.extend_from_slice(nonce.as_slice());
        output_data.extend_from_slice(&ciphertext);

        self.write_file(output_path, &output_data)
    }

    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        let data = self.read_file(input_path)?;
        
        if data.len() < 64 {
            return Err("Invalid encrypted file format".to_string());
        }

        let (key, nonce, ciphertext) = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(&data[0..32]);
                let nonce = Nonce::from_slice(&data[32..44]);
                (key, nonce, &data[44..])
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&data[0..32]);
                let nonce = ChaChaNonce::from_slice(&data[32..44]);
                (key, nonce, &data[44..])
            }
        };

        let plaintext = match self.algorithm {
            CipherAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(key);
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| format!("AES decryption failed: {}", e))?
            }
            CipherAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(key);
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|e| format!("ChaCha20 decryption failed: {}", e))?
            }
        };

        self.write_file(output_path, &plaintext)
    }

    fn aes_encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| format!("AES encryption failed: {}", e))?;
        
        Ok((ciphertext, key.to_vec(), nonce.to_vec()))
    }

    fn chacha_encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| format!("ChaCha20 encryption failed: {}", e))?;
        
        Ok((ciphertext, key.to_vec(), nonce.to_vec()))
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        let mut file = fs::File::open(path)
            .map_err(|e| format!("Failed to open file {}: {}", path, e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
        
        Ok(buffer)
    }

    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), String> {
        let path_obj = Path::new(path);
        if let Some(parent) = path_obj.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut file = fs::File::create(path)
            .map_err(|e| format!("Failed to create file {}: {}", path, e))?;
        
        file.write_all(data)
            .map_err(|e| format!("Failed to write file {}: {}", path, e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::Aes256Gcm);
        let test_data = b"Hello, World! This is a test message.";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_chacha_encryption_decryption() {
        let encryptor = FileEncryptor::new(CipherAlgorithm::ChaCha20Poly1305);
        let test_data = b"Another test with different algorithm";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        encryptor.encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap()
        ).unwrap();

        encryptor.decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap()
        ).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}