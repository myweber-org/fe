
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LENGTH],
    pub iv: [u8; IV_LENGTH],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(password: &str, input_path: &Path) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_file(password: &str, result: &EncryptionResult) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &result.salt);
    
    let plaintext = Aes256CbcDec::new(&key.into(), &result.iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&result.ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

pub fn save_encrypted_data(output_path: &Path, result: &EncryptionResult) -> Result<(), String> {
    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    file.write_all(&result.salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    file.write_all(&result.iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    file.write_all(&result.ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn load_encrypted_data(input_path: &Path) -> Result<EncryptionResult, String> {
    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    if data.len() < SALT_LENGTH + IV_LENGTH {
        return Err("File too short to contain valid encrypted data".to_string());
    }

    let salt = data[0..SALT_LENGTH].try_into().unwrap();
    let iv = data[SALT_LENGTH..SALT_LENGTH + IV_LENGTH].try_into().unwrap();
    let ciphertext = data[SALT_LENGTH + IV_LENGTH..].to_vec();

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}