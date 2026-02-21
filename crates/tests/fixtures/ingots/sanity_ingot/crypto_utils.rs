
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

pub fn generate_secure_token() -> String {
    let mut rng = thread_rng();
    let token: String = (0..32)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_token_format() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}