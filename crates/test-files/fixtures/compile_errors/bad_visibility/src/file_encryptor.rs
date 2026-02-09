
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

pub fn generate_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..KEY_LENGTH).map(|_| rng.gen()).collect()
}

pub fn generate_iv() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..IV_LENGTH).map(|_| rng.gen()).collect()
}

pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8], iv: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }
    if iv.len() != IV_LENGTH {
        return Err(format!("IV must be {} bytes", IV_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let ciphertext = Aes256CbcEnc::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8], iv: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!("Key must be {} bytes", KEY_LENGTH));
    }
    if iv.len() != IV_LENGTH {
        return Err(format!("IV must be {} bytes", IV_LENGTH));
    }

    let mut file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
    
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    let decrypted = Aes256CbcDec::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    output_file.write_all(&decrypted)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

pub fn save_key_iv(key: &[u8], iv: &[u8], path: &str) -> Result<(), String> {
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Failed to create key file: {}", e))?;
    
    let key_hex = hex::encode(key);
    let iv_hex = hex::encode(iv);
    let content = format!("KEY={}\nIV={}\n", key_hex, iv_hex);
    
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write key file: {}", e))?;
    
    Ok(())
}

pub fn load_key_iv(path: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    
    let mut key = None;
    let mut iv = None;
    
    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("KEY=") {
            key = Some(hex::decode(stripped)
                .map_err(|e| format!("Invalid key hex: {}", e))?);
        } else if let Some(stripped) = line.strip_prefix("IV=") {
            iv = Some(hex::decode(stripped)
                .map_err(|e| format!("Invalid IV hex: {}", e))?);
        }
    }
    
    match (key, iv) {
        (Some(k), Some(i)) => Ok((k, i)),
        _ => Err("Key file missing KEY or IV".to_string())
    }
}