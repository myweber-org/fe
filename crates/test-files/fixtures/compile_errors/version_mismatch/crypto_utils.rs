
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    generate_random_string(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let result = generate_random_string(10);
        assert_eq!(result.len(), 10);
        assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}