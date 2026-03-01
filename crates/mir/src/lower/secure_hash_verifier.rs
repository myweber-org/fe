
use ring::digest::{Context, SHA256};
use ring::constant_time::verify_slices_equal;

pub fn compute_sha256(data: &[u8]) -> Vec<u8> {
    let mut context = Context::new(&SHA256);
    context.update(data);
    context.finish().as_ref().to_vec()
}

pub fn verify_hash_constant_time(expected: &[u8], actual: &[u8]) -> bool {
    verify_slices_equal(expected, actual).is_ok()
}

pub struct HashVerifier {
    algorithm: &'static str,
}

impl HashVerifier {
    pub fn new() -> Self {
        HashVerifier {
            algorithm: "SHA256",
        }
    }

    pub fn verify_data(&self, data: &[u8], expected_hash: &[u8]) -> bool {
        let computed_hash = compute_sha256(data);
        verify_hash_constant_time(&computed_hash, expected_hash)
    }

    pub fn verify_file_hash<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        expected_hash: &[u8],
    ) -> std::io::Result<bool> {
        let data = std::fs::read(path)?;
        Ok(self.verify_data(&data, expected_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let data = b"test data";
        let hash = compute_sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_constant_time_verification() {
        let hash1 = compute_sha256(b"data1");
        let hash2 = compute_sha256(b"data2");
        let hash3 = compute_sha256(b"data1");

        assert!(!verify_hash_constant_time(&hash1, &hash2));
        assert!(verify_hash_constant_time(&hash1, &hash3));
    }

    #[test]
    fn test_verifier_integration() {
        let verifier = HashVerifier::new();
        let data = b"secure payload";
        let hash = compute_sha256(data);

        assert!(verifier.verify_data(data, &hash));
        assert!(!verifier.verify_data(b"tampered data", &hash));
    }
}