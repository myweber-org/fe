use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct FileEncryptor {
    cipher: Aes256Gcm,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Self {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt);
        let cipher = Aes256Gcm::new(&key);
        Self { cipher }
    }

    fn generate_salt() -> [u8; SALT_SIZE] {
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();
        *Key::<Aes256Gcm>::from_slice(&result)
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;
        
        let mut plaintext = Vec::new();
        file.read_to_end(&mut plaintext)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

        Ok(())
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        let mut file = File::open(input_path)
            .map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .map_err(|e| format!("Failed to read encrypted data: {}", e))?;

        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        output.write_all(&plaintext)
            .map_err(|e| format!("Failed to write decrypted data: {}", e))?;

        Ok(())
    }
}

pub fn process_directory(dir_path: &Path, password: &str, encrypt: bool) -> Result<(), String> {
    if !dir_path.is_dir() {
        return Err("Provided path is not a directory".to_string());
    }

    let encryptor = FileEncryptor::new(password);

    for entry in fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension()
                .unwrap_or_default()
                .to_string_lossy();
            
            if extension != "enc" && encrypt {
                let encrypted_path = path.with_extension("enc");
                encryptor.encrypt_file(&path, &encrypted_path)?;
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove original file: {}", e))?;
            } else if extension == "enc" && !encrypt {
                let decrypted_path = path.with_extension("");
                encryptor.decrypt_file(&path, &decrypted_path)?;
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove encrypted file: {}", e))?;
            }
        }
    }

    Ok(())
}use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_KEY: u8 = 0x55;

pub fn encrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    let encryption_key = key.unwrap_or(DEFAULT_KEY);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    for byte in buffer.iter_mut() {
        *byte ^= encryption_key;
    }

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&buffer)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, key: Option<u8>) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

pub fn process_directory(dir_path: &str, operation: &str, key: Option<u8>) -> io::Result<()> {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let input_str = path.to_str().unwrap();
            let output_str = format!("{}.processed", input_str);
            
            match operation {
                "encrypt" => encrypt_file(input_str, &output_str, key)?,
                "decrypt" => decrypt_file(input_str, &output_str, key)?,
                _ => return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid operation. Use 'encrypt' or 'decrypt'",
                )),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_data = b"Test data for encryption";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(original_data).unwrap();

        let input_path = temp_file.path().to_str().unwrap();
        let encrypted_path = format!("{}.enc", input_path);
        let decrypted_path = format!("{}.dec", input_path);

        encrypt_file(input_path, &encrypted_path, Some(0xAA)).unwrap();
        decrypt_file(&encrypted_path, &decrypted_path, Some(0xAA)).unwrap();

        let decrypted_data = fs::read(decrypted_path).unwrap();
        assert_eq!(original_data.to_vec(), decrypted_data);
    }
}