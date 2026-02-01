
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use indicatif::{ProgressBar, ProgressStyle};

pub struct FileHasher {
    chunk_size: usize,
}

impl FileHasher {
    pub fn new(chunk_size: usize) -> Self {
        FileHasher { chunk_size }
    }

    pub fn calculate_sha256<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();
        
        let pb = ProgressBar::new(file_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        let mut hasher = Sha256::new();
        let mut reader = io::BufReader::new(file);
        let mut buffer = vec![0; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            pb.inc(bytes_read as u64);
        }

        pb.finish_with_message("Hashing completed");
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_file<P: AsRef<Path>>(&self, path: P, expected_hash: &str) -> io::Result<bool> {
        let calculated = self.calculate_sha256(path)?;
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
            hash_string("hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_file_hashing() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Sample content for hashing")?;
        
        let hasher = FileHasher::new(8192);
        let hash = hasher.calculate_sha256(temp_file.path())?;
        
        assert_eq!(hash.len(), 64);
        Ok(())
    }

    #[test]
    fn test_file_verification() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Verification test data")?;
        
        let hasher = FileHasher::new(4096);
        let hash = hasher.calculate_sha256(temp_file.path())?;
        
        assert!(hasher.verify_file(temp_file.path(), &hash)?);
        assert!(!hasher.verify_file(temp_file.path(), "invalid_hash")?);
        Ok(())
    }
}