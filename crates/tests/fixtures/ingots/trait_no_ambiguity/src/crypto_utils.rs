use sha2::{Digest, Sha256};
use rand::{thread_rng, RngCore};

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut buffer = vec![0u8; len];
    rng.fill_bytes(&mut buffer);
    buffer
}

pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn sha256_hex_string(data: &[u8]) -> String {
    let hash = sha256_hash(data);
    hex::encode(hash)
}