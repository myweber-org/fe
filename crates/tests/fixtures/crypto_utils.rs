
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
    let prefix = "sk_live_";
    let random_part: String = generate_secure_token(32);
    format!("{}{}", prefix, random_part)
}

pub fn generate_session_token() -> String {
    generate_secure_token(64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_length() {
        let token = generate_secure_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_api_key_format() {
        let api_key = generate_api_key();
        assert!(api_key.starts_with("sk_live_"));
        assert_eq!(api_key.len(), 40);
    }

    #[test]
    fn test_unique_tokens() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();
        assert_ne!(token1, token2);
    }
}