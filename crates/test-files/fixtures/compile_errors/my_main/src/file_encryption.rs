
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use pbkdf2::{
    password_hash::{
        rand_core::RngCore,
        PasswordHasher, SaltString
    },
    Params, Pbkdf2
};
use std::fs;

pub struct FileEncryptor {
    key: [u8; 32],
    salt: SaltString,
}

impl FileEncryptor {
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params {
            rounds: 100_000,
            output_length: 32,
        };
        
        let password_hash = Pbkdf2.hash_password_customized(
            password.as_bytes(),
            None,
            None,
            params,
            &salt
        )?;
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self { key, salt })
    }
    
    pub fn encrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let encrypted_data = cipher.encrypt(Nonce::from_slice(&nonce), data.as_ref())?;
        
        let mut output = Vec::new();
        output.extend_from_slice(self.salt.as_str().as_bytes());
        output.push(b'|');
        output.extend_from_slice(&nonce);
        output.push(b'|');
        output.extend_from_slice(&encrypted_data);
        
        fs::write(output_path, output)?;
        Ok(())
    }
    
    pub fn decrypt_file(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read(input_path)?;
        let parts: Vec<&[u8]> = data.split(|&b| b == b'|').collect();
        
        if parts.len() != 3 {
            return Err("Invalid encrypted file format".into());
        }
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Nonce::from_slice(parts[1]);
        
        let decrypted_data = cipher.decrypt(nonce, parts[2])?;
        
        fs::write(output_path, decrypted_data)?;
        Ok(())
    }
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}