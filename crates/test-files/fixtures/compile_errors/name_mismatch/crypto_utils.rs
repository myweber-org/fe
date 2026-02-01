use rand::Rng;
use sha2::{Sha256, Digest};

pub fn generate_salt() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_password(password: &str, salt: &[u8], expected_hash: &str) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == expected_hash
}