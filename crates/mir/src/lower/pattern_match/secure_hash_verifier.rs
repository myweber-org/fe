
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use sha2::{Sha256, Digest};
use indicatif::{ProgressBar, ProgressStyle};

pub struct FileHasher {
    chunk_size: usize,
}

impl FileHasher {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub fn calculate_sha256<P: AsRef<Path>>(&self, file_path: P) -> Result<String, std::io::Error> {
        let file = File::open(file_path)?;
        let file_size = file.metadata()?.len();
        
        let mut hasher = Sha256::new();
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; self.chunk_size];
        
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );

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
}

pub fn verify_file_integrity<P: AsRef<Path>>(
    file_path: P,
    expected_hash: &str
) -> Result<bool, std::io::Error> {
    let hasher = FileHasher::new(8192);
    let calculated_hash = hasher.calculate_sha256(file_path)?;
    
    Ok(calculated_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hasher = FileHasher::new(4096);
        let hash = hasher.calculate_sha256(temp_file.path()).unwrap();
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test data").unwrap();
        
        let hasher = FileHasher::new(4096);
        let hash = hasher.calculate_sha256(temp_file.path()).unwrap();
        
        assert!(verify_file_integrity(temp_file.path(), &hash).unwrap());
    }
}