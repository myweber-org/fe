use ring::constant_time::verify_slices_are_equal;
use ring::digest;

pub fn compare_hashes(hash1: &[u8], hash2: &[u8]) -> bool {
    if hash1.len() != hash2.len() {
        return false;
    }
    
    match verify_slices_are_equal(hash1, hash2) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut context = digest::Context::new(&digest::SHA256);
    context.update(salt);
    context.update(password.as_bytes());
    context.finish().as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::{SecureRandom, SystemRandom};

    #[test]
    fn test_hash_comparison() {
        let rng = SystemRandom::new();
        let mut salt = [0u8; 16];
        rng.fill(&mut salt).unwrap();

        let password = "correct_password";
        let wrong_password = "wrong_password";

        let correct_hash = hash_password(password, &salt);
        let wrong_hash = hash_password(wrong_password, &salt);

        assert!(compare_hashes(&correct_hash, &correct_hash));
        assert!(!compare_hashes(&correct_hash, &wrong_hash));
    }

    #[test]
    fn test_different_lengths() {
        let hash1 = vec![1, 2, 3];
        let hash2 = vec![1, 2, 3, 4];
        
        assert!(!compare_hashes(&hash1, &hash2));
    }
}