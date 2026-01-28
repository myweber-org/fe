
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_password(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_password(length: usize) -> String {
    let mut rng = thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let char_type: u8 = rng.gen_range(0..4);
        let c = match char_type {
            0 => rng.gen_range(b'a'..=b'z') as char,
            1 => rng.gen_range(b'A'..=b'Z') as char,
            2 => rng.gen_range(b'0'..=b'9') as char,
            _ => {
                let symbols = "!@#$%^&*()-_=+[]{}|;:,.<>?";
                symbols.chars().nth(rng.gen_range(0..symbols.len())).unwrap()
            }
        };
        password.push(c);
    }
    
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        let password = generate_password(12);
        assert_eq!(password.len(), 12);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_secure_password() {
        let password = generate_secure_password(16);
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_numeric()));
    }
}