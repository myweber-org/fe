
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_salt(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    hasher.finalize().to_vec()
}

pub fn verify_password(password: &str, salt: &[u8], hash: &[u8]) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == hash
}