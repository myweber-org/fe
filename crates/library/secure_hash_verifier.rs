
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher;

pub enum HashAlgorithm {
    SHA256,
    BLAKE3,
}

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_file_hash<P: AsRef<Path>>(
        path: P,
        algorithm: HashAlgorithm
    ) -> Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let hash = match algorithm {
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::BLAKE3 => {
                let mut hasher = Hasher::new();
                hasher.update(&buffer);
                hasher.finalize().to_string()
            }
        };

        Ok(hash)
    }

    pub fn verify_file_hash<P: AsRef<Path>>(
        path: P,
        expected_hash: &str,
        algorithm: HashAlgorithm
    ) -> Result<bool> {
        let calculated_hash = Self::calculate_file_hash(path, algorithm)?;
        Ok(calculated_hash == expected_hash)
    }

    pub fn compare_files<P: AsRef<Path>>(
        file1: P,
        file2: P,
        algorithm: HashAlgorithm
    ) -> Result<bool> {
        let hash1 = Self::calculate_file_hash(file1, algorithm)?;
        let hash2 = Self::calculate_file_hash(file2, algorithm)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_hashing() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test content for hashing")?;
        
        let hash = HashVerifier::calculate_file_hash(
            temp_file.path(),
            HashAlgorithm::SHA256
        )?;
        
        assert_eq!(hash.len(), 64);
        Ok(())
    }

    #[test]
    fn test_blake3_hashing() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Another test content")?;
        
        let hash = HashVerifier::calculate_file_hash(
            temp_file.path(),
            HashAlgorithm::BLAKE3
        )?;
        
        assert!(!hash.is_empty());
        Ok(())
    }

    #[test]
    fn test_file_comparison() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        writeln!(file1, "Same content")?;
        writeln!(file2, "Same content")?;
        
        let are_equal = HashVerifier::compare_files(
            file1.path(),
            file2.path(),
            HashAlgorithm::SHA256
        )?;
        
        assert!(are_equal);
        Ok(())
    }
}