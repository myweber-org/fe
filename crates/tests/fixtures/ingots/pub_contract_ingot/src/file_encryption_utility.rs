
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        let key_bytes = password_hash.hash.ok_or("Hash generation failed")?;
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = fs::read(input_path)?;
        
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output_data.extend_from_slice(&nonce);
        output_data.extend_from_slice(&ciphertext);
        
        fs::write(output_path, output_data)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_data = fs::read(input_path)?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, plaintext)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("File Encryption Utility");
    println!("=======================");
    
    print!("Enter password: ");
    io::stdout().flush()?;
    
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let encryptor = FileEncryptor::from_password(password)?;
    
    println!("\nOptions:");
    println!("1. Encrypt file");
    println!("2. Decrypt file");
    print!("Select option (1/2): ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => {
            print!("Input file path: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input_path = input.trim();
            
            print!("Output file path: ");
            io::stdout().flush()?;
            let mut output = String::new();
            io::stdin().read_line(&mut output)?;
            let output_path = output.trim();
            
            encryptor.encrypt_file(input_path, output_path)?;
            println!("File encrypted successfully!");
        }
        "2" => {
            print!("Input file path: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input_path = input.trim();
            
            print!("Output file path: ");
            io::stdout().flush()?;
            let mut output = String::new();
            io::stdin().read_line(&mut output)?;
            let output_path = output.trim();
            
            encryptor.decrypt_file(input_path, output_path)?;
            println!("File decrypted successfully!");
        }
        _ => println!("Invalid option"),
    }
    
    Ok(())
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, String> {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];
        
        argon2.hash_password_into(
            password.as_bytes(),
            salt,
            &mut key
        ).map_err(|e| format!("Key derivation failed: {}", e))?;
        
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        Ok(FileEncryptor { cipher })
    }
    
    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?
            .read_to_end(&mut file_data)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut rng = OsRng;
        let nonce_bytes: [u8; NONCE_SIZE] = rng.random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, file_data.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let mut output_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&ciphertext);
        
        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&output_data)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;
        
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut encrypted_data = Vec::new();
        fs::File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?
            .read_to_end(&mut encrypted_data)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;
        
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        let ciphertext = &encrypted_data[NONCE_SIZE..];
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?
            .write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;
        
        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut rng = OsRng;
    rng.random()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let salt = generate_salt();
        
        let encryptor = FileEncryptor::from_password(password, &salt)
            .expect("Failed to create encryptor");
        
        let original_content = b"Secret data that needs protection";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        encryptor.encrypt_file(input_file.path(), encrypted_file.path())
            .expect("Encryption failed");
        
        let encryptor2 = FileEncryptor::from_password(password, &salt)
            .expect("Failed to recreate encryptor");
        
        encryptor2.decrypt_file(encrypted_file.path(), decrypted_file.path())
            .expect("Decryption failed");
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Key::<Aes256Gcm>::generate(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;
    output_file.write_all(&key)?;

    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short to contain key",
        ));
    }

    let (ciphertext, key_bytes) = encrypted_data.split_at(encrypted_data.len() - 32);
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Secret data that needs protection";
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}