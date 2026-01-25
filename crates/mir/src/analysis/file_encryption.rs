use std::fs;
use std::io::{self, Read, Write};

pub struct XORCipher {
    key: Vec<u8>,
}

impl XORCipher {
    pub fn new(key: &str) -> Self {
        XORCipher {
            key: key.as_bytes().to_vec(),
        }
    }

    pub fn encrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    pub fn decrypt_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        self.process_file(source_path, dest_path)
    }

    fn process_file(&self, source_path: &str, dest_path: &str) -> io::Result<()> {
        let mut source_file = fs::File::open(source_path)?;
        let mut buffer = Vec::new();
        source_file.read_to_end(&mut buffer)?;

        let processed_data: Vec<u8> = buffer
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.key[i % self.key.len()])
            .collect();

        let mut dest_file = fs::File::create(dest_path)?;
        dest_file.write_all(&processed_data)?;

        Ok(())
    }
}

pub fn generate_random_key(length: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_xor_cipher() {
        let test_data = b"Hello, XOR encryption!";
        let key = "secret_key";
        let cipher = XORCipher::new(key);

        let encrypted: Vec<u8> = test_data
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key.as_bytes()[i % key.len()])
            .collect();

        let decrypted: Vec<u8> = encrypted
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key.as_bytes()[i % key.len()])
            .collect();

        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_file_encryption() {
        let test_content = "Test file content for encryption";
        let test_file = "test_input.txt";
        let encrypted_file = "test_encrypted.bin";
        let decrypted_file = "test_decrypted.txt";

        fs::write(test_file, test_content).unwrap();

        let cipher = XORCipher::new("test_password");
        cipher.encrypt_file(test_file, encrypted_file).unwrap();
        cipher.decrypt_file(encrypted_file, decrypted_file).unwrap();

        let decrypted_content = fs::read_to_string(decrypted_file).unwrap();
        assert_eq!(test_content, decrypted_content);

        fs::remove_file(test_file).unwrap();
        fs::remove_file(encrypted_file).unwrap();
        fs::remove_file(decrypted_file).unwrap();
    }
}