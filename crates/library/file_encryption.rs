
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};
use std::fs;
use std::io::{self, Read, Write};

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext)?;

    let salt = SaltString::generate(&mut OsRng);
    let key_material = Pbkdf2.hash_password(password.as_bytes(), &salt)?;
    let key_hash = key_material.ok_or("Key derivation failed")?;
    let key_bytes = key_hash.hash.ok_or("No hash generated")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&OsRng.gen::<[u8; NONCE_LENGTH]>());

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = fs::File::create(output_path)?;
    output.write_all(salt.as_str().as_bytes())?;
    output.write_all(&nonce)?;
    output.write_all(&ciphertext)?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(input_path)?;
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)?;

    if encrypted_data.len() < SALT_LENGTH + NONCE_LENGTH {
        return Err("Invalid encrypted file format".into());
    }

    let salt_bytes = &encrypted_data[..SALT_LENGTH];
    let salt = SaltString::from_b64(std::str::from_utf8(salt_bytes)?)?;
    let nonce = Nonce::from_slice(&encrypted_data[SALT_LENGTH..SALT_LENGTH + NONCE_LENGTH]);
    let ciphertext = &encrypted_data[SALT_LENGTH + NONCE_LENGTH..];

    let key_material = Pbkdf2.hash_password(password.as_bytes(), &salt)?;
    let key_hash = key_material.ok_or("Key derivation failed")?;
    let key_bytes = key_hash.hash.ok_or("No hash generated")?;

    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::write(output_path, plaintext)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <encrypt|decrypt> <input> <output> <password>", args[0]);
        std::process::exit(1);
    }

    let operation = &args[1];
    let input = &args[2];
    let output = &args[3];
    let password = &args[4];

    match operation.as_str() {
        "encrypt" => encrypt_file(input, output, password)?,
        "decrypt" => decrypt_file(input, output, password)?,
        _ => {
            eprintln!("Invalid operation. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }

    println!("Operation completed successfully");
    Ok(())
}