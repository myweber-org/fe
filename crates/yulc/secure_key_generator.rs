
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_key(length: usize) -> String {
    let mut rng = thread_rng();
    
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_api_key() -> String {
    generate_secure_key(32)
}

pub fn generate_session_token() -> String {
    generate_secure_key(64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length() {
        let key = generate_secure_key(16);
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_api_key_length() {
        let key = generate_api_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_session_token_length() {
        let token = generate_session_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_unique_keys() {
        let key1 = generate_secure_key(16);
        let key2 = generate_secure_key(16);
        assert_ne!(key1, key2);
    }
}