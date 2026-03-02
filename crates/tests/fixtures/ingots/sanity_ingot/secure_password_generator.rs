use rand::rngs::OsRng;
use rand::Rng;
use std::iter;

pub fn generate_password(length: usize, include_uppercase: bool, include_numbers: bool, include_symbols: bool) -> String {
    let mut charset = "abcdefghijklmnopqrstuvwxyz".to_string();
    
    if include_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if include_numbers {
        charset.push_str("0123456789");
    }
    if include_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    let charset_bytes: Vec<u8> = charset.bytes().collect();
    
    let password: String = iter::repeat(())
        .map(|()| {
            let idx = OsRng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .take(length)
        .collect();

    password
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
    fn test_password_charset() {
        let password = generate_password(20, false, false, false);
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_password_uniqueness() {
        let pass1 = generate_password(16, true, true, true);
        let pass2 = generate_password(16, true, true, true);
        assert_ne!(pass1, pass2);
    }
}