
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    let mut rng = thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(16);
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_token_format() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
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
    fn test_generate_token_length() {
        let token = generate_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_hash_password_consistency() {
        let password = "secure_password";
        let salt = "random_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_different_salts() {
        let password = "same_password";
        let hash1 = hash_password(password, "salt1");
        let hash2 = hash_password(password, "salt2");
        assert_ne!(hash1, hash2);
    }
}