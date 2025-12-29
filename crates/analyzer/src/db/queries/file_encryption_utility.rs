use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

const BUFFER_SIZE: usize = 8192;

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

    pub fn encrypt(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.key[self.key_position];
            self.key_position = (self.key_position + 1) % self.key.len();
        }
    }

    pub fn reset(&mut self) {
        self.key_position = 0;
    }
}

pub fn process_file(input_path: &Path, output_path: &Path, key: &str) -> io::Result<()> {
    let mut cipher = XorCipher::new(key);
    let mut input_file = fs::File::open(input_path)?;
    let mut output_file = fs::File::create(output_path)?;

    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let data_slice = &mut buffer[..bytes_read];
        cipher.encrypt(data_slice);
        output_file.write_all(data_slice)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xor_cipher_symmetry() {
        let original_data = b"Hello, World!";
        let mut data = original_data.to_vec();
        let key = "secret_key";

        let mut cipher = XorCipher::new(key);
        cipher.encrypt(&mut data);
        
        cipher.reset();
        cipher.encrypt(&mut data);

        assert_eq!(data, original_data);
    }

    #[test]
    fn test_file_encryption() -> io::Result<()> {
        let test_content = b"Test file content for encryption";
        let key = "test_key_123";

        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        fs::write(input_file.path(), test_content)?;

        process_file(input_file.path(), output_file.path(), key)?;

        let mut cipher = XorCipher::new(key);
        let mut encrypted_content = fs::read(output_file.path())?;
        cipher.encrypt(&mut encrypted_content);

        assert_eq!(encrypted_content, test_content);
        Ok(())
    }
}