
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let data = fs::read(input_path)?;
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(&nonce_bytes)?;
    output.write_all(&ciphertext)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain valid encrypted data"
        ));
    }
    
    let key = &encrypted_data[..32];
    let nonce_bytes = &encrypted_data[32..32 + NONCE_SIZE];
    let ciphertext = &encrypted_data[32 + NONCE_SIZE..];
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, plaintext)?;
    Ok(())
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let encrypted_data = xor_encrypt(&input_data, key);
    fs::write(output_path, encrypted_data)
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let input_data = fs::read(input_path)?;
    let decrypted_data = xor_decrypt(&input_data, key);
    fs::write(output_path, decrypted_data)
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key)
}

pub fn process_files() -> io::Result<()> {
    let key = b"secret_key";
    let original_file = "document.txt";
    let encrypted_file = "document.enc";
    let decrypted_file = "document_decrypted.txt";

    fs::write(original_file, b"Confidential data: Project details")?;
    
    xor_encrypt_file(original_file, encrypted_file, key)?;
    xor_decrypt_file(encrypted_file, decrypted_file, key)?;

    let original_content = fs::read_to_string(original_file)?;
    let decrypted_content = fs::read_to_string(decrypted_file)?;

    assert_eq!(original_content, decrypted_content);
    println!("Encryption and decryption successful!");
    
    Ok(())
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::io::{self, Read, Write};

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut file = fs::File::open(input_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    if contents.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too short",
        ));
    }

    let (key_bytes, rest) = contents.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    Ok(())
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{pbkdf2_hmac, Params};
use rand::RngCore;
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut input_file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext).map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let mut key = [0u8; KEY_LEN];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LEN,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, params.rounds, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file.write_all(&salt).map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file.write_all(&iv).map_err(|e| format!("Failed to write IV: {}", e))?;
    output_file.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut input_file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data).map_err(|e| format!("Failed to read input file: {}", e))?;

    if encrypted_data.len() < SALT_LEN + IV_LEN {
        return Err("File too short to contain salt and IV".to_string());
    }

    let (salt_data, rest) = encrypted_data.split_at(SALT_LEN);
    let (iv_data, ciphertext) = rest.split_at(IV_LEN);

    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    salt.copy_from_slice(salt_data);
    iv.copy_from_slice(iv_data);

    let mut key = [0u8; KEY_LEN];
    let params = Params {
        rounds: KEY_ITERATIONS,
        output_length: KEY_LEN,
    };
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, params.rounds, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    let decrypted = Aes256CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let mut output_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file.write_all(&decrypted).map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(())
}

pub fn encrypt_directory(dir_path: &str, password: &str) -> Result<(), String> {
    for entry in fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            let input_path = path.to_str().ok_or("Invalid file path")?;
            let output_path = format!("{}.enc", input_path);
            encrypt_file(input_path, &output_path, password)?;
            fs::remove_file(input_path).map_err(|e| format!("Failed to remove original file: {}", e))?;
        }
    }
    Ok(())
}

pub fn decrypt_directory(dir_path: &str, password: &str) -> Result<(), String> {
    for entry in fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "enc" {
                    let input_path = path.to_str().ok_or("Invalid file path")?;
                    let output_path = input_path.trim_end_matches(".enc");
                    decrypt_file(input_path, output_path, password)?;
                    fs::remove_file(input_path).map_err(|e| format!("Failed to remove encrypted file: {}", e))?;
                }
            }
        }
    }
    Ok(())
}