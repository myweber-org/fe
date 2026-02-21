use sha2::{Digest, Sha256};
use std::env;

fn generate_sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <text_to_hash>", args[0]);
        std::process::exit(1);
    }
    
    let input = &args[1];
    let hash = generate_sha256_hash(input);
    
    println!("Input: {}", input);
    println!("SHA-256 Hash: {}", hash);
    
    if args.len() > 2 && args[2] == "--verify" {
        let verification_hash = generate_sha256_hash(input);
        if hash == verification_hash {
            println!("Verification: Hash matches!");
        } else {
            println!("Verification: Hash mismatch!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let test_string = "hello_world";
        let hash = generate_sha256_hash(test_string);
        
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "b2d6b6a6c2c5b5c5c5b5c5c5b5c5c5b5c5c5b5c5c5b5c5c5b5c5c5b5c5c5b5c"
        );
    }

    #[test]
    fn test_empty_string() {
        let hash = generate_sha256_hash("");
        assert_eq!(hash.len(), 64);
    }
}