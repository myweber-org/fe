use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_hash<P: AsRef<Path>>(
        path: P,
        algorithm: HashAlgorithm,
    ) -> Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let hash = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(&buffer);
                format!("{}", hasher.finalize())
            }
        };

        Ok(hash)
    }

    pub fn verify_hash<P: AsRef<Path>>(
        path: P,
        expected_hash: &str,
        algorithm: HashAlgorithm,
    ) -> Result<bool> {
        let calculated_hash = Self::calculate_hash(path, algorithm)?;
        Ok(calculated_hash == expected_hash)
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
        temp_file.write_all(b"test data")?;
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Sha256,
        )?;
        
        let expected = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";
        assert_eq!(hash, expected);
        
        let verified = FileHashVerifier::verify_hash(
            temp_file.path(),
            expected,
            HashAlgorithm::Sha256,
        )?;
        assert!(verified);
        
        Ok(())
    }

    #[test]
    fn test_blake3_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"test data")?;
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Blake3,
        )?;
        
        let verified = FileHashVerifier::verify_hash(
            temp_file.path(),
            &hash,
            HashAlgorithm::Blake3,
        )?;
        assert!(verified);
        
        Ok(())
    }
}