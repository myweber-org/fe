
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_secure_token() -> String {
    generate_random_string(32)
}

pub fn generate_api_key() -> String {
    let prefix = "api_";
    let suffix = generate_random_string(24);
    format!("{}{}", prefix, suffix)
}