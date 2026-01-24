use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// XOR cipher implementation for file encryption/decryption
pub struct XorCipher {
    key: Vec<u8>,
}

impl XorCipher {
    /// Create new cipher with given key
    pub fn new(key: &[u8]) -> Self {
        XorCipher { key: key.to_vec() }
    }

    /// Process data using XOR cipher
    pub fn process(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect()
    }
}

/// Encrypt file using XOR cipher
pub fn encrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    let cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let encrypted_data = cipher.process(&buffer);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;

    Ok(())
}

/// Decrypt file using XOR cipher
pub fn decrypt_file(input_path: &Path, output_path: &Path, key: &[u8]) -> io::Result<()> {
    encrypt_file(input_path, output_path, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let key = b"secret_key";
        let cipher = XorCipher::new(key);
        let original_data = b"Hello, World! This is test data.";

        let encrypted = cipher.process(original_data);
        let decrypted = cipher.process(&encrypted);

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let key = b"test_key_123";
        let original_content = b"Sample file content for encryption test.";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        fs::write(input_file.path(), original_content)?;

        encrypt_file(input_file.path(), output_file.path(), key)?;

        let encrypted_content = fs::read(output_file.path())?;
        assert_ne!(original_content, encrypted_content.as_slice());

        let decrypted_file = NamedTempFile::new()?;
        decrypt_file(output_file.path(), decrypted_file.path(), key)?;

        let decrypted_content = fs::read(decrypted_file.path())?;
        assert_eq!(original_content, decrypted_content.as_slice());

        Ok(())
    }
}