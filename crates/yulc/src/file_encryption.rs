use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::error::Error;

pub fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = Vec::with_capacity(key.len() + ciphertext.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if ciphertext.len() < 32 {
        return Err("Invalid ciphertext length".into());
    }
    
    let key = Key::<Aes256Gcm>::from_slice(&ciphertext[..32]);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    
    cipher.decrypt(nonce, &ciphertext[32..])
        .map_err(|e| format!("Decryption failed: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let original = b"secret message";
        let encrypted = encrypt_data(original).unwrap();
        let decrypted = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original.to_vec(), decrypted);
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{pbkdf2_hmac, Params};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    key: Key<Aes256Gcm>,
}

impl FileEncryptor {
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let mut key = [0u8; 32];
        let params = Params {
            rounds: PBKDF2_ITERATIONS,
            output_length: 32,
        };
        
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, params.rounds, &mut key);
        
        FileEncryptor {
            key: Key::<Aes256Gcm>::from_slice(&key).into(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);
        
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(&[0u8; NONCE_LENGTH]);
        
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output_file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output_file.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn generate_salt() -> [u8; SALT_LENGTH] {
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn encrypt_with_password(
    password: &str,
    input_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let salt = generate_salt();
    let encryptor = FileEncryptor::from_password(password, &salt);
    
    let mut salted_output = salt.to_vec();
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    encryptor.encrypt_file(input_path, temp_file.path())?;
    
    let mut encrypted_data = Vec::new();
    File::open(temp_file.path())
        .and_then(|mut f| f.read_to_end(&mut encrypted_data))
        .map_err(|e| format!("Failed to read temp file: {}", e))?;
    
    salted_output.extend_from_slice(&encrypted_data);
    
    fs::write(output_path, &salted_output)
        .map_err(|e| format!("Failed to write final output: {}", e))?;
    
    Ok(())
}

pub fn decrypt_with_password(
    password: &str,
    input_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if data.len() < SALT_LENGTH {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let (salt, encrypted_data) = data.split_at(SALT_LENGTH);
    let encryptor = FileEncryptor::from_password(password, salt);
    
    let temp_input = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    fs::write(temp_input.path(), encrypted_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    encryptor.decrypt_file(temp_input.path(), output_path)
}use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::error::Error;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn generate_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill(&mut key);
    key
}

pub fn generate_iv() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut iv = [0u8; 16];
    rng.fill(&mut iv);
    iv
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<String, Box<dyn Error>> {
    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    Ok(hex::encode(ciphertext))
}

pub fn decrypt_data(ciphertext_hex: &str, key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>, Box<dyn Error>> {
    let ciphertext = hex::decode(ciphertext_hex)?;
    let plaintext = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let iv = generate_iv();
        let original_data = b"Secret message for encryption test";

        let encrypted = encrypt_data(original_data, &key, &iv).unwrap();
        let decrypted = decrypt_data(&encrypted, &key, &iv).unwrap();

        assert_eq!(original_data.to_vec(), decrypted);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    
    let input_data = fs::read(input_path)?;
    let encrypted_data: Vec<u8> = input_data
        .iter()
        .map(|byte| byte ^ encryption_key)
        .collect();
    
    fs::write(output_path, encrypted_data)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_file_interactive() -> io::Result<()> {
    println!("Enter input file path:");
    let mut input_path = String::new();
    io::stdin().read_line(&mut input_path)?;
    let input_path = input_path.trim();
    
    println!("Enter output file path:");
    let mut output_path = String::new();
    io::stdin().read_line(&mut output_path)?;
    let output_path = output_path.trim();
    
    println!("Encrypt (e) or Decrypt (d)?");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    println!("Enter encryption key (0-255, press Enter for default):");
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    
    let key = if key_input.trim().is_empty() {
        None
    } else {
        match key_input.trim().parse::<u8>() {
            Ok(k) => Some(k),
            Err(_) => {
                eprintln!("Invalid key, using default");
                None
            }
        }
    };
    
    match choice.trim() {
        "e" | "E" => encrypt_file(input_path, output_path, key),
        "d" | "D" => decrypt_file(input_path, output_path, key),
        _ => {
            eprintln!("Invalid choice");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Hello, World!";
        let test_key = Some(0xAA);
        
        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_data).unwrap();
        
        encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            test_key
        ).unwrap();
        
        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}