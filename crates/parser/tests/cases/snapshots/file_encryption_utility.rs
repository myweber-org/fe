
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

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
    pub salt: String,
}

pub fn derive_key(password: &str, salt: &str) -> Result<Key<Aes256Gcm>, Box<dyn std::error::Error>> {
    let salt_bytes = SaltString::from_b64(salt)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_bytes)?;
    
    let hash_bytes = password_hash.hash.ok_or("Hash generation failed")?;
    let key_bytes: [u8; 32] = hash_bytes.as_bytes()[..32].try_into()?;
    
    Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let plaintext = fs::read(input_path)?;
    
    let salt = SaltString::generate(&mut OsRng);
    let key = derive_key(password, salt.as_str())?;
    
    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, &ciphertext)?;
    
    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes,
        salt: salt.to_string(),
    })
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
    salt: &str,
    nonce: &[u8; NONCE_SIZE]
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let ciphertext = fs::read(input_path)?;
    
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, &plaintext)?;
    
    Ok(plaintext)
}

pub fn interactive_encrypt() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter input file path: ");
    io::stdout().flush()?;
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    print!("Enter output file path: ");
    io::stdout().flush()?;
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    print!("Enter encryption password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    let result = encrypt_file(input_path, output_path, password)?;
    
    println!("Encryption successful!");
    println!("Salt: {}", result.salt);
    println!("Nonce: {}", hex::encode(result.nonce));
    println!("Ciphertext saved to: {}", output_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encrypt_decrypt_cycle() {
        let test_data = b"Test encryption data for AES-256-GCM";
        let password = "secure_password_123";
        
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(test_data).unwrap();
        
        let output_encrypted = NamedTempFile::new().unwrap();
        let output_decrypted = NamedTempFile::new().unwrap();
        
        let encrypt_result = encrypt_file(
            input_file.path().to_str().unwrap(),
            output_encrypted.path().to_str().unwrap(),
            password
        ).unwrap();
        
        let decrypted_data = decrypt_file(
            output_encrypted.path().to_str().unwrap(),
            output_decrypted.path().to_str().unwrap(),
            password,
            &encrypt_result.salt,
            &encrypt_result.nonce
        ).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}