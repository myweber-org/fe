use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let mut rng = thread_rng();
    let password: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let length = 16;
        let password = generate_password(length);
        assert_eq!(password.len(), length);
    }

    #[test]
    fn test_password_content() {
        let password = generate_password(20);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_unique_passwords() {
        let pass1 = generate_password(12);
        let pass2 = generate_password(12);
        assert_ne!(pass1, pass2);
    }
}