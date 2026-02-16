
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_token_charset() {
        let token = generate_token(100);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_hash_consistency() {
        let hash1 = hash_password("mypassword", "somesalt");
        let hash2 = hash_password("mypassword", "somesalt");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_salts() {
        let hash1 = hash_password("mypassword", "salt1");
        let hash2 = hash_password("mypassword", "salt2");
        assert_ne!(hash1, hash2);
    }
}