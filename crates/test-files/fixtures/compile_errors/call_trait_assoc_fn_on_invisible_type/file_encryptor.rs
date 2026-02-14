
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

pub fn encrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    
    let data = fs::read(input_path)?;
    let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);
    
    let encrypted_data = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output = fs::File::create(output_path)?;
    output.write_all(&key)?;
    output.write_all(nonce.as_slice())?;
    output.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let encrypted_data = fs::read(input_path)?;
    
    if encrypted_data.len() < 32 + NONCE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain valid encrypted data",
        ));
    }
    
    let key = &encrypted_data[..32];
    let nonce = &encrypted_data[32..32 + NONCE_SIZE];
    let ciphertext = &encrypted_data[32 + NONCE_SIZE..];
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let decrypted_data = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    fs::write(output_path, decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_encrypt_decrypt_cycle() {
        let original_content = b"Test encryption and decryption data";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), original_content).unwrap();
        
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        encrypt_file(input_file.path(), encrypted_file.path()).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}