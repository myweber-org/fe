
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use std::fs;

pub fn encrypt_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let encrypted_data = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    fs::write(output_path, encrypted_data)?;
    fs::write(format!("{}.key", output_path), key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let key_data = fs::read(key_path)?;
    let key = key_data.as_slice().try_into()
        .map_err(|_| "Invalid key length")?;
    
    let cipher = Aes256Gcm::new(&key);
    let encrypted_data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct FileCrypto;

impl FileCrypto {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut rng = rand::thread_rng();
        
        let mut salt = [0u8; SALT_LEN];
        rng.fill_bytes(&mut salt);
        
        let mut iv = [0u8; IV_LEN];
        rng.fill_bytes(&mut iv);
        
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);
        
        let plaintext = fs::read(input_path).map_err(|e| e.to_string())?;
        
        let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);
        
        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + ciphertext.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&ciphertext);
        
        fs::write(output_path, &output).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let data = fs::read(input_path).map_err(|e| e.to_string())?;
        
        if data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let salt = &data[0..SALT_LEN];
        let iv = &data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &data[SALT_LEN + IV_LEN..];
        
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        
        let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| e.to_string())?;
        
        fs::write(output_path, &plaintext).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    pub fn generate_key_file(path: &str, password: &str) -> Result<(), String> {
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; SALT_LEN];
        rng.fill_bytes(&mut salt);
        
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, KEY_ITERATIONS, &mut key);
        
        let mut output = Vec::with_capacity(SALT_LEN + KEY_LEN);
        output.extend_from_slice(&salt);
        output.extend_from_slice(&key);
        
        fs::write(path, &output).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}