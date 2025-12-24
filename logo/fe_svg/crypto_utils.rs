
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; len];
    rng.fill_bytes(&mut buffer);
    buffer
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn generate_secure_token() -> String {
    let random_bytes = generate_random_bytes(32);
    let hash = sha256_hash(&random_bytes);
    hex::encode(hash)
}