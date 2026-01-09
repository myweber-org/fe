
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    key: [u8; 32],
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> io::Result<Self> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Salt error: {}", e))
        })?;
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidInput, format!("Hashing error: {}", e))
            })?;
        
        let hash_bytes = password_hash.hash.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "No hash generated")
        })?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        
        Ok(Self { key })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut file_data)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(&generate_random_bytes(NONCE_SIZE));
        
        let encrypted_data = cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {}", e)))?;
        
        let mut output = fs::File::create(output_path)?;
        output.write_all(nonce)?;
        output.write_all(&encrypted_data)?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)?.read_to_end(&mut encrypted_data)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let decrypted_data = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Decryption failed: {}", e)))?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
    
    pub fn generate_salt() -> [u8; SALT_SIZE] {
        generate_random_bytes(SALT_SIZE)
    }
}

fn generate_random_bytes(size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn process_encryption(
    password: &str,
    input_file: &str,
    output_file: &str,
    encrypt: bool,
) -> io::Result<()> {
    let salt_path = format!("{}.salt", output_file);
    let salt = if encrypt {
        let salt = FileEncryptor::generate_salt();
        fs::write(&salt_path, &salt)?;
        salt.to_vec()
    } else {
        fs::read(&salt_path)?
    };
    
    let encryptor = FileEncryptor::from_password(password, &salt)?;
    
    if encrypt {
        encryptor.encrypt_file(Path::new(input_file), Path::new(output_file))?;
        println!("Encryption completed successfully");
    } else {
        encryptor.decrypt_file(Path::new(input_file), Path::new(output_file))?;
        println!("Decryption completed successfully");
    }
    
    Ok(())
}