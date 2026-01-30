
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

pub struct FileCipher;

impl FileCipher {
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let mut file_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);
        
        let key = Self::derive_key(password, &salt);
        
        let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
        let encrypted_data = cipher.encrypt_padded_vec_mut::<Pkcs7>(&mut file_data);
        
        let mut output = Vec::with_capacity(SALT_LEN + IV_LEN + encrypted_data.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&iv);
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, &output).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
        let encrypted_data = fs::read(input_path).map_err(|e| format!("Read error: {}", e))?;
        
        if encrypted_data.len() < SALT_LEN + IV_LEN {
            return Err("Invalid encrypted file format".to_string());
        }
        
        let salt = &encrypted_data[0..SALT_LEN];
        let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
        let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];
        
        let key = Self::derive_key(password, salt);
        
        let cipher = Aes256CbcDec::new(&key.into(), iv.into());
        let decrypted_data = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        fs::write(output_path, &decrypted_data).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
    
    fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
        key
    }
}

pub fn process_files() -> Result<(), String> {
    let test_data = b"Secret data that needs protection";
    let input_file = "test_input.bin";
    let encrypted_file = "test_encrypted.bin";
    let decrypted_file = "test_decrypted.bin";
    let password = "strong_password_123";
    
    fs::write(input_file, test_data).map_err(|e| format!("Test setup failed: {}", e))?;
    
    FileCipher::encrypt_file(input_file, encrypted_file, password)?;
    FileCipher::decrypt_file(encrypted_file, decrypted_file, password)?;
    
    let restored_data = fs::read(decrypted_file).map_err(|e| format!("Verification failed: {}", e))?;
    
    if test_data.as_ref() != restored_data {
        return Err("Data corruption detected".to_string());
    }
    
    fs::remove_file(input_file).ok();
    fs::remove_file(encrypted_file).ok();
    fs::remove_file(decrypted_file).ok();
    
    println!("File encryption test completed successfully");
    Ok(())
}