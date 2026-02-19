use ring::constant_time::verify_slices_are_equal;
use ring::digest::{Context, SHA256};

pub fn compute_sha256(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    context.finish().as_ref().to_vec()
}

pub fn verify_hash(data: &[u8], expected_hash: &[u8]) -> Result<(), &'static str> {
    let computed_hash = compute_sha256(data);
    
    match verify_slices_are_equal(&computed_hash, expected_hash) {
        Ok(_) => Ok(()),
        Err(_) => Err("Hash verification failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_verification() {
        let test_data = b"secure payload";
        let correct_hash = compute_sha256(test_data);
        let wrong_hash = compute_sha256(b"different payload");

        assert!(verify_hash(test_data, &correct_hash).is_ok());
        assert!(verify_hash(test_data, &wrong_hash).is_err());
    }

    #[test]
    fn test_constant_time_comparison() {
        let hash1 = vec![0u8; 32];
        let hash2 = vec![1u8; 32];
        let hash3 = vec![0u8; 32];

        assert!(verify_slices_are_equal(&hash1, &hash2).is_err());
        assert!(verify_slices_are_equal(&hash1, &hash3).is_ok());
    }
}