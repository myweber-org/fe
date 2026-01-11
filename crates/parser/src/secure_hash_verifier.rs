
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};

pub fn verify_file_hash(file_path: &str, expected_hash: &str) -> Result<bool> {
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

    let computed_hash = hasher.finalize();
    let computed_hash_hex = hex::encode(computed_hash);
    
    Ok(computed_hash_hex == expected_hash.to_lowercase())
}

pub fn compute_file_hash(file_path: &str) -> Result<String> {
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

    let computed_hash = hasher.finalize();
    Ok(hex::encode(computed_hash))
}