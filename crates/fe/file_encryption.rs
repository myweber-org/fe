
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

fn xor_crypt(data: &mut [u8], key: u8) {
    for byte in data.iter_mut() {
        *byte ^= key;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    let key = key.unwrap_or(DEFAULT_KEY);
    let mut content = fs::read(input_path)?;
    xor_crypt(&mut content, key);
    fs::write(output_path, content)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_crypt() {
        let mut data = vec![0x00, 0xFF, 0xAA, 0x55];
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0xAA, 0x55, 0x00, 0xFF]);
        xor_crypt(&mut data, 0xAA);
        assert_eq!(data, vec![0x00, 0xFF, 0xAA, 0x55]);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let original = b"Secret data";
        let input_temp = NamedTempFile::new()?;
        fs::write(input_temp.path(), original)?;

        let output_temp = NamedTempFile::new()?;
        encrypt_file(input_temp.path(), output_temp.path(), Some(0x42))?;

        let encrypted = fs::read(output_temp.path())?;
        assert_ne!(encrypted, original);

        let decrypt_temp = NamedTempFile::new()?;
        decrypt_file(output_temp.path(), decrypt_temp.path(), Some(0x42))?;
        let decrypted = fs::read(decrypt_temp.path())?;
        assert_eq!(decrypted, original);

        Ok(())
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{self, Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const KEY_LENGTH: usize = 32;
const IV_LENGTH: usize = 16;

fn generate_key() -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    rand::thread_rng().fill(&mut key);
    key
}

fn generate_iv() -> [u8; IV_LENGTH] {
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill(&mut iv);
    iv
}

fn encrypt_file(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    let key = generate_key();
    let iv = generate_iv();

    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&key)?;
    output_file.write_all(&iv)?;
    output_file.write_all(&ciphertext)?;

    println!("Encryption successful. Key: {}", hex::encode(key));
    println!("IV: {}", hex::encode(iv));
    println!("Output saved to: {}", output_path);

    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key_hex: &str, iv_hex: &str) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)?;

    let key = hex::decode(key_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let iv = hex::decode(iv_hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if key.len() != KEY_LENGTH || iv.len() != IV_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid key or IV length",
        ));
    }

    let key_arr: [u8; KEY_LENGTH] = key.try_into().unwrap();
    let iv_arr: [u8; IV_LENGTH] = iv.try_into().unwrap();

    let plaintext = Aes256CbcDec::new(&key_arr.into(), &iv_arr.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&plaintext)?;

    println!("Decryption successful. Output saved to: {}", output_path);

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!("  Encrypt: {} encrypt <input> <output>", args[0]);
        eprintln!("  Decrypt: {} decrypt <input> <output> <key_hex> <iv_hex>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match mode.as_str() {
        "encrypt" => {
            if let Err(e) = encrypt_file(input_path, output_path) {
                eprintln!("Encryption failed: {}", e);
                std::process::exit(1);
            }
        }
        "decrypt" => {
            if args.len() < 6 {
                eprintln!("Decrypt requires key and IV in hex format");
                std::process::exit(1);
            }
            let key_hex = &args[4];
            let iv_hex = &args[5];
            if let Err(e) = decrypt_file(input_path, output_path, key_hex, iv_hex) {
                eprintln!("Decryption failed: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Invalid mode. Use 'encrypt' or 'decrypt'");
            std::process::exit(1);
        }
    }
}
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const KEY_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

pub struct EncryptionResult {
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub ciphertext: Vec<u8>,
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, KEY_ITERATIONS, &mut key);
    key
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<EncryptionResult, String> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?
        .read_to_end(&mut file_data)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let key = derive_key(password, &salt);
    let ciphertext = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&file_data);

    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    output_file
        .write_all(&salt)
        .map_err(|e| format!("Failed to write salt: {}", e))?;
    output_file
        .write_all(&iv)
        .map_err(|e| format!("Failed to write IV: {}", e))?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

    Ok(EncryptionResult {
        salt,
        iv,
        ciphertext,
    })
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &str,
) -> Result<Vec<u8>, String> {
    let mut encrypted_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("Failed to open encrypted file: {}", e))?
        .read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    if encrypted_data.len() < SALT_LEN + IV_LEN {
        return Err("Encrypted file too short".to_string());
    }

    let salt = &encrypted_data[0..SALT_LEN];
    let iv = &encrypted_data[SALT_LEN..SALT_LEN + IV_LEN];
    let ciphertext = &encrypted_data[SALT_LEN + IV_LEN..];

    let key = derive_key(password, salt);
    let plaintext = Aes256CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?
        .write_all(&plaintext)
        .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let test_data = b"Hello, this is a secret message!";
        let password = "strong_password_123";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), test_data).unwrap();

        let enc_result = encrypt_file(input_file.path(), encrypted_file.path(), password).unwrap();
        assert_eq!(enc_result.salt.len(), SALT_LEN);
        assert_eq!(enc_result.iv.len(), IV_LEN);
        assert!(!enc_result.ciphertext.is_empty());

        let decrypted = decrypt_file(encrypted_file.path(), decrypted_file.path(), password).unwrap();
        assert_eq!(decrypted, test_data);

        let wrong_password_result = decrypt_file(encrypted_file.path(), decrypted_file.path(), "wrong_password");
        assert!(wrong_password_result.is_err());
    }
}
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    pbkdf2_hmac,
    Params
};
use sha2::Sha256;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> io::Result<Self> {
        let salt: [u8; SALT_LENGTH] = OsRng::default()
            .try_fill_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?[..SALT_LENGTH]
            .try_into()
            .unwrap();

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &salt,
            PBKDF2_ITERATIONS,
            &mut key,
            Params::default(),
        );

        let cipher_key = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(cipher_key);

        Ok(Self { cipher })
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let nonce: [u8; NONCE_LENGTH] = OsRng::default()
            .try_fill_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?[..NONCE_LENGTH]
            .try_into()
            .unwrap();

        let ciphertext = self.cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&nonce)?;
        output_file.write_all(&ciphertext)?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> io::Result<()> {
        let mut input_file = fs::File::open(input_path)?;
        let mut encrypted_data = Vec::new();
        input_file.read_to_end(&mut encrypted_data)?;

        if encrypted_data.len() < NONCE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too short to contain nonce",
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut output_file = fs::File::create(output_path)?;
        output_file.write_all(&plaintext)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let password = "secure_password_123";
        let encryptor = FileEncryptor::new(password).unwrap();

        let original_content = b"Secret data that needs protection";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_content).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encryptor
            .encrypt_file(input_file.path(), encrypted_file.path())
            .unwrap();
        encryptor
            .decrypt_file(encrypted_file.path(), decrypted_file.path())
            .unwrap();

        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())
            .unwrap()
            .read_to_end(&mut decrypted_content)
            .unwrap();

        assert_eq!(decrypted_content, original_content);
    }
}