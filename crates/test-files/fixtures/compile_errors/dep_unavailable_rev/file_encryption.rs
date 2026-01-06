
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
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
    
    let key_path = format!("{}.key", output_path);
    fs::write(key_path, key.as_slice())?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &str, key_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encrypted_data = fs::read(input_path)?;
    let key_data = fs::read(key_path)?;
    
    let key = key_data.as_slice().try_into()
        .map_err(|_| "Invalid key length")?;
    
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(b"unique_nonce_");
    
    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    fs::write(output_path, decrypted_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_encryption_roundtrip() {
        let test_data = b"Test encryption data";
        let input_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let key_file = "test_encrypted.bin.key";
        let decrypted_file = "test_decrypted.txt";
        
        fs::write(input_file, test_data).unwrap();
        
        encrypt_file(input_file, encrypted_file).unwrap();
        decrypt_file(encrypted_file, key_file, decrypted_file).unwrap();
        
        let decrypted = fs::read(decrypted_file).unwrap();
        assert_eq!(decrypted, test_data);
        
        fs::remove_file(input_file).ok();
        fs::remove_file(encrypted_file).ok();
        fs::remove_file(key_file).ok();
        fs::remove_file(decrypted_file).ok();
    }
}