
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    SHA256,
    BLAKE3,
}

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_hash<P: AsRef<Path>>(
        path: P,
        algorithm: HashAlgorithm,
    ) -> io::Result<String> {
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
                let mut hasher = Blake3Hasher::new();
                hasher.update(&buffer);
                hasher.finalize().to_string()
            }
        };

        Ok(hash)
    }

    pub fn verify_hash<P: AsRef<Path>>(
        path: P,
        expected_hash: &str,
        algorithm: HashAlgorithm,
    ) -> io::Result<bool> {
        let calculated_hash = Self::calculate_hash(path, algorithm)?;
        Ok(calculated_hash == expected_hash)
    }

    pub fn compare_files<P: AsRef<Path>>(
        path1: P,
        path2: P,
        algorithm: HashAlgorithm,
    ) -> io::Result<bool> {
        let hash1 = Self::calculate_hash(path1, algorithm)?;
        let hash2 = Self::calculate_hash(path2, algorithm)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test data for hashing").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::SHA256
        ).unwrap();
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_blake3_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "another test data").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::BLAKE3
        ).unwrap();
        
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_file_comparison() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        writeln!(file1, "identical content").unwrap();
        writeln!(file2, "identical content").unwrap();
        
        let result = FileHashVerifier::compare_files(
            file1.path(),
            file2.path(),
            HashAlgorithm::SHA256
        ).unwrap();
        
        assert!(result);
    }
}