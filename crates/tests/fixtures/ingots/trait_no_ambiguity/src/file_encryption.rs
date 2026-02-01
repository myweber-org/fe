
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn xor_encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; 1024];
    let key_len = key.len();
    let mut key_index = 0;

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        for byte in buffer.iter_mut().take(bytes_read) {
            *byte ^= key[key_index];
            key_index = (key_index + 1) % key_len;
        }

        output_file.write_all(&buffer[..bytes_read])?;
    }

    output_file.flush()?;
    Ok(())
}

pub fn xor_decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_encryption_decryption() {
        let key = b"secret_key";
        let original_data = b"Hello, this is a test message for XOR encryption!";

        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(original_data).unwrap();

        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        xor_encrypt_file(input_file.path(), encrypted_file.path(), key).unwrap();
        xor_decrypt_file(encrypted_file.path(), decrypted_file.path(), key).unwrap();

        let decrypted_data = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }
}
use std::fs;
use std::io::{self, Read, Write};

pub fn xor_encrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data: Vec<u8> = buffer
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect();

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn xor_decrypt_file(input_path: &str, output_path: &str, key: &[u8]) -> io::Result<()> {
    xor_encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let original_content = b"Hello, World! This is a test.";
        let key = b"secret_key";

        let input_file = NamedTempFile::new().unwrap();
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        fs::write(input_file.path(), original_content).unwrap();

        xor_encrypt_file(
            input_file.path().to_str().unwrap(),
            encrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        xor_decrypt_file(
            encrypted_file.path().to_str().unwrap(),
            decrypted_file.path().to_str().unwrap(),
            key,
        )
        .unwrap();

        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, true)
    }

    pub fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> Result<(), String> {
        self.process_file(input_path, output_path, false)
    }

    fn process_file(&self, input_path: &Path, output_path: &Path, is_encrypt: bool) -> Result<(), String> {
        if !input_path.exists() {
            return Err(format!("Input file does not exist: {:?}", input_path));
        }

        let mut input_file = fs::File::open(input_path)
            .map_err(|e| format!("Failed to open input file: {}", e))?;

        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read input file: {}", e))?;

        let processed_data = self.xor_transform(&buffer);

        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        output_file.write_all(&processed_data)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    fn xor_transform(&self, data: &[u8]) -> Vec<u8> {
        let key_len = self.key.len();
        if key_len == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % key_len])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher() {
        let cipher = XorCipher::new("secret_key");
        let test_data = b"Hello, World!";
        
        let encrypted = cipher.xor_transform(test_data);
        let decrypted = cipher.xor_transform(&encrypted);
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let cipher = XorCipher::new("test_key");
        let original_content = b"Sample file content for encryption test.";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), original_content).unwrap();
        
        cipher.encrypt_file(input_file.path(), output_file.path()).unwrap();
        cipher.decrypt_file(output_file.path(), decrypted_file.path()).unwrap();
        
        let decrypted_content = fs::read(decrypted_file.path()).unwrap();
        assert_eq!(original_content.to_vec(), decrypted_content);
    }
}
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct XorCipher {
    key: Vec<u8>,
    key_position: usize,
}

impl XorCipher {
    pub fn new(key: &str) -> Self {
        XorCipher {
            key: key.as_bytes().to_vec(),
            key_position: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        self.process(data)
    }

    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        
        for &byte in data {
            let key_byte = self.key[self.key_position];
            result.push(byte ^ key_byte);
            
            self.key_position = (self.key_position + 1) % self.key.len();
        }
        
        result
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let encrypted_data = cipher.encrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    
    Ok(())
}

pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    
    let decrypted_data = cipher.decrypt(&buffer);
    
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = "secret_key";
        let mut cipher = XorCipher::new(key);
        
        let original_data = b"Hello, World! This is a test message.";
        let encrypted = cipher.encrypt(original_data);
        
        cipher.reset();
        let decrypted = cipher.decrypt(&encrypted);
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = "test_key_123";
        let original_content = b"Sample file content for encryption test.";
        
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(original_content)?;
        
        let encrypted_file = NamedTempFile::new()?;
        let decrypted_file = NamedTempFile::new()?;
        
        encrypt_file(input_file.path(), encrypted_file.path(), key)?;
        decrypt_file(encrypted_file.path(), decrypted_file.path(), key)?;
        
        let mut decrypted_content = Vec::new();
        fs::File::open(decrypted_file.path())?
            .read_to_end(&mut decrypted_content)?;
        
        assert_eq!(original_content.to_vec(), decrypted_content);
        Ok(())
    }
}