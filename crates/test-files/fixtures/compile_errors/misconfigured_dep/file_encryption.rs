
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use rand::Rng;

const KEY_SIZE: usize = 32;

fn generate_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..KEY_SIZE).map(|_| rng.gen()).collect()
}

fn xor_operation(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .zip(key.iter().cycle())
        .map(|(d, k)| d ^ k)
        .collect()
}

fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> std::io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = xor_operation(&buffer, key);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> std::io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

fn save_key(key: &[u8], path: &Path) -> std::io::Result<()> {
    let mut key_file = fs::File::create(path)?;
    key_file.write_all(key)?;
    Ok(())
}

fn load_key(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut key_file = fs::File::open(path)?;
    let mut key = Vec::new();
    key_file.read_to_end(&mut key)?;
    Ok(key)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file> [key_file]", args[0]);
        std::process::exit(1);
    }
    
    let operation = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);
    
    let key = if args.len() > 4 {
        let key_path = Path::new(&args[4]);
        match load_key(key_path) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Failed to load key: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let new_key = generate_key();
        let key_path = Path::new("encryption_key.bin");
        if let Err(e) = save_key(&new_key, key_path) {
            eprintln!("Warning: Failed to save generated key: {}", e);
        }
        new_key
    };
    
    let result = match operation.as_str() {
        "encrypt" => encrypt_file(input_path, output_path, &key),
        "decrypt" => decrypt_file(input_path, output_path, &key),
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        eprintln!("Operation failed: {}", e);
        std::process::exit(1);
    }
    
    println!("Operation completed successfully");
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| e.to_string())?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32]
        .try_into()
        .map_err(|_| "Key derivation failed")?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let mut rng = OsRng;
    let mut nonce = [0u8; NONCE_SIZE];
    let mut salt = [0u8; SALT_SIZE];
    
    rng.fill_bytes(&mut nonce);
    rng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(&nonce);
    let encrypted_data = cipher
        .encrypt(nonce_obj, plaintext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&encrypted_data).map_err(|e| e.to_string())?;

    Ok(EncryptionResult {
        encrypted_data,
        nonce,
        salt,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
    nonce: &[u8; NONCE_SIZE],
    salt: &[u8; SALT_SIZE],
) -> Result<Vec<u8>, String> {
    let mut file = File::open(input_path).map_err(|e| e.to_string())?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext).map_err(|e| e.to_string())?;

    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_obj = Nonce::from_slice(nonce);
    let decrypted_data = cipher
        .decrypt(nonce_obj, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    let mut output_file = File::create(output_path).map_err(|e| e.to_string())?;
    output_file.write_all(&decrypted_data).map_err(|e| e.to_string())?;

    Ok(decrypted_data)
}

pub fn generate_random_key() -> [u8; 32] {
    let mut rng = OsRng;
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = b"Test encryption and decryption";
        let password = "secure_password_123";
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), plaintext).unwrap();
        
        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password)
            .expect("Encryption failed");
        
        let decrypted = decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            password,
            &enc_result.nonce,
            &enc_result.salt,
        ).expect("Decryption failed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);
        
        let key1 = derive_key(password, &salt).expect("Key derivation 1 failed");
        let key2 = derive_key(password, &salt).expect("Key derivation 2 failed");
        
        assert_eq!(key1.as_slice(), key2.as_slice());
    }
}