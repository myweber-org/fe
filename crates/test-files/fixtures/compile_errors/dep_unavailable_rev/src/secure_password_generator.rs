
use rand::rngs::OsRng;
use rand::RngCore;
use std::iter;

const PASSWORD_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*()-_=+[]{}|;:,.<>?";

pub fn generate_password(length: usize) -> String {
    let mut rng = OsRng;
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = (rng.next_u32() as usize) % PASSWORD_CHARS.len();
        password.push(PASSWORD_CHARS[idx] as char);
    }
    
    password
}

pub fn generate_secure_password(length: usize) -> Result<String, &'static str> {
    if length < 8 {
        return Err("Password length must be at least 8 characters");
    }
    
    let mut password = generate_password(length);
    
    // Ensure password contains at least one character from each category
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| 
        "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)
    );
    
    if !(has_uppercase && has_lowercase && has_digit && has_special) {
        // Regenerate if requirements aren't met
        password = generate_password(length);
    }
    
    Ok(password)
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
    fn test_secure_password_requirements() {
        let password = generate_secure_password(12).unwrap();
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| 
            "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)
        ));
    }
    
    #[test]
    fn test_invalid_length() {
        let result = generate_secure_password(6);
        assert!(result.is_err());
    }
}