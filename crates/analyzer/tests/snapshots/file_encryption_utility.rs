
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonRng, PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: [u8; SALT_SIZE],
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| format!("Salt encoding failed: {}", e))?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_str)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?.as_bytes();
    if hash_bytes.len() < 32 {
        return Err("Derived key too short".to_string());
    }
    
    let key_slice = &hash_bytes[..32];
    Ok(*Key::<Aes256Gcm>::from_slice(key_slice))
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut rng = OsRng;
    let mut nonce = [0u8; NONCE_SIZE];
    let mut salt = [0u8; SALT_SIZE];
    
    rng.fill_bytes(&mut nonce);
    rng.fill_bytes(&mut salt);
    
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;
    
    Ok(EncryptionResult {
        ciphertext,
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
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read ciphertext: {}", e))?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&plaintext)
        .map_err(|e| format!("Failed to write plaintext: {}", e))?;
    
    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), String> {
    println!("Enter input file path:");
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input.trim());
    
    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin().read_line(&mut output)
        .map_err(|e| format!("Failed to read output: {}", e))?;
    let output_path = Path::new(output.trim());
    
    println!("Enter encryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    
    let result = encrypt_file(input_path, output_path, password.trim())?;
    
    println!("Encryption successful!");
    println!("Nonce (hex): {}", hex::encode(result.nonce));
    println!("Salt (hex): {}", hex::encode(result.salt));
    println!("Save these values for decryption.");
    
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), String> {
    println!("Enter encrypted file path:");
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    let input_path = Path::new(input.trim());
    
    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin().read_line(&mut output)
        .map_err(|e| format!("Failed to read output: {}", e))?;
    let output_path = Path::new(output.trim());
    
    println!("Enter decryption password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password)
        .map_err(|e| format!("Failed to read password: {}", e))?;
    
    println!("Enter nonce (hex):");
    let mut nonce_hex = String::new();
    io::stdin().read_line(&mut nonce_hex)
        .map_err(|e| format!("Failed to read nonce: {}", e))?;
    let nonce_bytes = hex::decode(nonce_hex.trim())
        .map_err(|e| format!("Invalid hex for nonce: {}", e))?;
    if nonce_bytes.len() != NONCE_SIZE {
        return Err(format!("Nonce must be {} bytes", NONCE_SIZE));
    }
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.copy_from_slice(&nonce_bytes);
    
    println!("Enter salt (hex):");
    let mut salt_hex = String::new();
    io::stdin().read_line(&mut salt_hex)
        .map_err(|e| format!("Failed to read salt: {}", e))?;
    let salt_bytes = hex::decode(salt_hex.trim())
        .map_err(|e| format!("Invalid hex for salt: {}", e))?;
    if salt_bytes.len() != SALT_SIZE {
        return Err(format!("Salt must be {} bytes", SALT_SIZE));
    }
    let mut salt = [0u8; SALT_SIZE];
    salt.copy_from_slice(&salt_bytes);
    
    decrypt_file(input_path, output_path, password.trim(), &nonce, &salt)?;
    
    println!("Decryption successful!");
    
    Ok(())
}