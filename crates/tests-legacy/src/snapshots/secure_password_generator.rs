use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;

#[derive(Debug)]
pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn set_length(&mut self, length: usize) -> &mut Self {
        if length < 4 {
            panic!("Password length must be at least 4 characters");
        }
        self.length = length;
        self
    }

    pub fn use_uppercase(&mut self, enable: bool) -> &mut Self {
        self.use_uppercase = enable;
        self
    }

    pub fn use_lowercase(&mut self, enable: bool) -> &mut Self {
        self.use_lowercase = enable;
        self
    }

    pub fn use_digits(&mut self, enable: bool) -> &mut Self {
        self.use_digits = enable;
        self
    }

    pub fn use_special(&mut self, enable: bool) -> &mut Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> String {
        let mut character_pool = Vec::new();

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            panic!("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        password
    }
}

fn main() {
    let mut generator = PasswordGenerator::new();

    println!("Password Generator");
    println!("==================");

    let length = get_user_input("Enter password length (default 16): ");
    if let Ok(len) = length.trim().parse::<usize>() {
        generator.set_length(len);
    }

    let uppercase = get_user_input("Include uppercase letters? (Y/n): ");
    if uppercase.trim().to_lowercase() == "n" {
        generator.use_uppercase(false);
    }

    let lowercase = get_user_input("Include lowercase letters? (Y/n): ");
    if lowercase.trim().to_lowercase() == "n" {
        generator.use_lowercase(false);
    }

    let digits = get_user_input("Include digits? (Y/n): ");
    if digits.trim().to_lowercase() == "n" {
        generator.use_digits(false);
    }

    let special = get_user_input("Include special characters? (Y/n): ");
    if special.trim().to_lowercase() == "n" {
        generator.use_special(false);
    }

    let password = generator.generate();
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input
}