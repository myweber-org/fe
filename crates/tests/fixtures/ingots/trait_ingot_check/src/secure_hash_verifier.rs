
use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};
use blake3::Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_hash(file_path: &str, algorithm: HashAlgorithm) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let hash = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Hasher::new();
                hasher.update(&buffer);
                hasher.finalize().to_string()
            }
        };

        Ok(hash)
    }

    pub fn verify_hash(file_path: &str, expected_hash: &str, algorithm: HashAlgorithm) -> Result<bool> {
        let calculated_hash = Self::calculate_hash(file_path, algorithm)?;
        Ok(calculated_hash == expected_hash)
    }

    pub fn compare_files(file1: &str, file2: &str, algorithm: HashAlgorithm) -> Result<bool> {
        let hash1 = Self::calculate_hash(file1, algorithm)?;
        let hash2 = Self::calculate_hash(file2, algorithm)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test content for hash verification")?;
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path().to_str().unwrap(),
            HashAlgorithm::Sha256
        )?;
        
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
        Ok(())
    }

    #[test]
    fn test_blake3_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Another test content")?;
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path().to_str().unwrap(),
            HashAlgorithm::Blake3
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
        
        let are_equal = FileHashVerifier::compare_files(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            HashAlgorithm::Sha256
        )?;
        
        assert!(are_equal);
        Ok(())
    }
}
use std::fs::File;
use std::io::{Read, self};
use sha2::{Sha256, Digest};

pub fn compute_file_hash(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_integrity(original_path: &str, comparison_path: &str) -> io::Result<bool> {
    let original_hash = compute_file_hash(original_path)?;
    let comparison_hash = compute_file_hash(comparison_path)?;
    
    Ok(original_hash == comparison_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> io::Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        let result = verify_file_integrity(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        )?;
        
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_different_files() -> io::Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"First test data")?;
        file2.write_all(b"Second test data")?;
        
        let result = verify_file_integrity(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        )?;
        
        assert!(!result);
        Ok(())
    }
}