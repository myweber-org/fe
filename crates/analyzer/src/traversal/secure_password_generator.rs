use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = "abcdefghijklmnopqrstuvwxyz".to_string();
    
    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

pub fn generate_secure_password(length: usize) -> String {
    generate_password(length, true, true, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12, true, true, true);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_character_sets() {
        let password = generate_password(20, false, false, false);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_secure_password() {
        let password = generate_secure_password(16);
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_numeric()));
        assert!(password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)));
    }
}