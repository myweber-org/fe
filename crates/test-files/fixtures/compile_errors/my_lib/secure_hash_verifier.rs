
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