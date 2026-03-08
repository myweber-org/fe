
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use indicatif::{ProgressBar, ProgressStyle};

pub fn calculate_file_hash(file_path: &Path) -> io::Result<String> {
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();
    
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        pb.inc(bytes_read as u64);
    }
    
    pb.finish_with_message("Hash calculation complete");
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> io::Result<bool> {
    let calculated_hash = calculate_file_hash(file_path)?;
    Ok(calculated_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = calculate_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test").unwrap();
        
        let hash = calculate_file_hash(temp_file.path()).unwrap();
        let is_valid = verify_file_integrity(temp_file.path(), &hash).unwrap();
        
        assert!(is_valid);
    }
}
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_file_integrity<P: AsRef<Path>>(
        file_path: P,
        expected_hash: &str
    ) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files<P: AsRef<Path>>(
        file1: P,
        file2: P
    ) -> Result<bool, Error> {
        let hash1 = Self::calculate_sha256(file1)?;
        let hash2 = Self::calculate_sha256(file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        assert!(FileHashVerifier::compare_files(
            file1.path(),
            file2.path()
        )?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Data 1")?;
        file2.write_all(b"Data 2")?;
        
        assert!(!FileHashVerifier::compare_files(
            file1.path(),
            file2.path()
        )?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<(), Error> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Test content")?;
        
        let hash = FileHashVerifier::calculate_sha256(file.path())?;
        let is_valid = FileHashVerifier::verify_file_integrity(
            file.path(),
            &hash
        )?;
        
        assert!(is_valid);
        Ok(())
    }
}