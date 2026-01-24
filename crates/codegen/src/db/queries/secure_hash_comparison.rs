use subtle::{Choice, ConstantTimeEq};

pub fn verify_hash(provided_hash: &[u8], stored_hash: &[u8]) -> bool {
    if provided_hash.len() != stored_hash.len() {
        return false;
    }
    
    let mut result = Choice::from(1);
    for (a, b) in provided_hash.iter().zip(stored_hash.iter()) {
        result &= a.ct_eq(b);
    }
    
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_matching_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_different_hashes() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("f3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert!(!verify_hash(&hash1, &hash2));
    }

    #[test]
    fn test_different_lengths() {
        let hash1 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash2 = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85");
        assert!(!verify_hash(&hash1, &hash2));
    }
}