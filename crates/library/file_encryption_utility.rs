
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; 32],
) -> Result<EncryptionResult, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = OsRng.fill(&mut OsRng);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&ciphertext)?;

    Ok(EncryptionResult {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; 32],
    nonce: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(plaintext)
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill(&mut OsRng);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Test encryption data";
        let key = generate_key();

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        let result = encrypt_file(input_file.path(), encrypted_file.path(), &key).unwrap();
        let nonce = result.nonce;

        decrypt_file(
            encrypted_file.path(),
            decrypted_file.path(),
            &key,
            &nonce,
        )
        .unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(test_data.to_vec(), decrypted_data);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let plaintext = fs::read(input_path)?;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_material = password_hash.hash.ok_or("Hash generation failed")?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut output_data = Vec::new();
    output_data.extend_from_slice(salt.as_ref());
    output_data.extend_from_slice(&nonce_bytes);
    output_data.extend_from_slice(&ciphertext);
    
    fs::write(output_path, output_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 32 {
        return Err("Invalid encrypted file format".into());
    }
    
    let salt = SaltString::from_b64(&String::from_utf8_lossy(&encrypted_data[..22]))?;
    let nonce_start = 22;
    let ciphertext_start = nonce_start + NONCE_SIZE;
    
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_material = password_hash.hash.ok_or("Hash generation failed")?;
    
    let key = Key::<Aes256Gcm>::from_slice(&key_material.as_bytes()[..32]);
    let cipher = Aes256Gcm::new(key);
    
    let nonce = Nonce::from_slice(&encrypted_data[nonce_start..ciphertext_start]);
    let ciphertext = &encrypted_data[ciphertext_start..];
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
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
    
    print!("Enter password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    encrypt_file(input_path, output_path, password)?;
    println!("File encrypted successfully");
    Ok(())
}

pub fn interactive_decrypt() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter encrypted file path: ");
    io::stdout().flush()?;
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    print!("Enter output file path: ");
    io::stdout().flush()?;
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    print!("Enter password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    decrypt_file(input_path, output_path, password)?;
    println!("File decrypted successfully");
    Ok(())
}