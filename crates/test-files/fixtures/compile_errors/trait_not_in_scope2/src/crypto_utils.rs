
use rand::Rng;

pub fn generate_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    password
}

pub fn generate_secure_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    rand::thread_rng().fill(&mut token);
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
    fn test_token_size() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 32);
    }
}
use rand::Rng;

pub fn generate_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    password
}

pub fn generate_secure_token() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut token = [0u8; 32];
    rng.fill(&mut token);
    token
}