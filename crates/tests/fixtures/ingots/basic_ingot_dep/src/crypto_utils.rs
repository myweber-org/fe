use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_secure_token(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_api_key() -> String {
    let token = generate_secure_token(32);
    format!("sk_{}", token)
}

pub fn generate_session_id() -> String {
    generate_secure_token(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_secure_token(24);
        assert_eq!(token.len(), 24);
    }

    #[test]
    fn test_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("sk_"));
        assert_eq!(key.len(), 35); // "sk_" + 32 chars
    }

    #[test]
    fn test_unique_tokens() {
        let token1 = generate_secure_token(16);
        let token2 = generate_secure_token(16);
        assert_ne!(token1, token2);
    }
}