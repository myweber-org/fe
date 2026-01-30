use subtle::ConstantTimeEq;

pub fn compare_hashes(hash1: &[u8], hash2: &[u8]) -> bool {
    if hash1.len() != hash2.len() {
        return false;
    }
    
    hash1.ct_eq(hash2).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_identical_hashes() {
        let hash_a = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash_b = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        
        assert!(compare_hashes(&hash_a, &hash_b));
    }

    #[test]
    fn test_different_hashes() {
        let hash_a = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash_b = hex!("d3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        
        assert!(!compare_hashes(&hash_a, &hash_b));
    }

    #[test]
    fn test_different_lengths() {
        let hash_a = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        let hash_b = hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85");
        
        assert!(!compare_hashes(&hash_a, &hash_b));
    }
}