
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};

pub struct FileVerifier {
    buffer_size: usize,
}

impl FileVerifier {
    pub fn new(buffer_size: usize) -> Self {
        FileVerifier { buffer_size }
    }

    pub fn calculate_hash<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            total_read += bytes_read;
            
            if total_read % (10 * 1024 * 1024) == 0 {
                eprintln!("Processed {} MB", total_read / (1024 * 1024));
            }
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_file<P: AsRef<Path>>(&self, path: P, expected_hash: &str) -> io::Result<bool> {
        let calculated = self.calculate_hash(path)?;
        Ok(calculated == expected_hash.to_lowercase())
    }
}

pub fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hashing() {
        let hash = hash_string("test_data");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash_string(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_file_verification() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"sample content for hashing")?;
        
        let verifier = FileVerifier::new(4096);
        let hash = verifier.calculate_hash(temp_file.path())?;
        
        assert!(verifier.verify_file(temp_file.path(), &hash)?);
        assert!(!verifier.verify_file(temp_file.path(), "invalid_hash")?);
        
        Ok(())
    }

    #[test]
    fn test_empty_file() -> io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let verifier = FileVerifier::new(1024);
        let hash = verifier.calculate_hash(temp_file.path())?;
        
        assert_eq!(hash, hash_string(""));
        Ok(())
    }
}