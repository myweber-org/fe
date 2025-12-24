
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use std::error::Error;

type HmacSha256 = Hmac<Sha256>;

pub struct HashVerifier {
    secret_key: Vec<u8>,
}

impl HashVerifier {
    pub fn new(secret_key: &[u8]) -> Self {
        Self {
            secret_key: secret_key.to_vec(),
        }
    }

    pub fn compute_sha256(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    pub fn compute_hmac(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    pub fn verify_hmac(&self, data: &[u8], expected_hmac: &[u8]) -> Result<bool, Box<dyn Error>> {
        let computed_hmac = self.compute_hmac(data)?;
        Ok(computed_hmac == expected_hmac)
    }

    pub fn hash_to_hex(&self, hash: &[u8]) -> String {
        hash.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_consistency() {
        let verifier = HashVerifier::new(b"test_key");
        let data = b"hello world";
        let hash1 = verifier.compute_sha256(data);
        let hash2 = verifier.compute_sha256(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hmac_verification() {
        let verifier = HashVerifier::new(b"secret_key");
        let data = b"message to authenticate";
        
        let hmac = verifier.compute_hmac(data).unwrap();
        let verification = verifier.verify_hmac(data, &hmac).unwrap();
        
        assert!(verification);
    }

    #[test]
    fn test_hmac_tamper_detection() {
        let verifier = HashVerifier::new(b"secret_key");
        let data = b"original message";
        let tampered_data = b"tampered message";
        
        let original_hmac = verifier.compute_hmac(data).unwrap();
        let verification = verifier.verify_hmac(tampered_data, &original_hmac).unwrap();
        
        assert!(!verification);
    }
}