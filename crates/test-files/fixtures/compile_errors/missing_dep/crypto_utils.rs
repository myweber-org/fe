use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    generate_random_string(32)
}use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = thread_rng();
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
    fn test_random_string_length() {
        let random = generate_random_string(16);
        assert_eq!(random.len(), 16);
    }

    #[test]
    fn test_hash_consistency() {
        let password = "secure_password";
        let salt = "random_salt";
        let hash1 = hash_password(password, salt);
        let hash2 = hash_password(password, salt);
        assert_eq!(hash1, hash2);
    }
}