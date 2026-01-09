
use rand::Rng;
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 16;

pub fn generate_salt() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut salt = vec![0u8; SALT_LENGTH];
    rng.fill(&mut salt[..]);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

pub fn verify_password(password: &str, salt: &[u8], stored_hash: &[u8]) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == stored_hash
}