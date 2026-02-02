use rand::Rng;
use std::io::{self, Write};

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_upper: bool,
    use_lower: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordConfig {
    fn new() -> Self {
        Self {
            length: 16,
            use_upper: true,
            use_lower: true,
            use_digits: true,
            use_special: true,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.length < 8 {
            return Err("Password length must be at least 8 characters".to_string());
        }
        
        let mut char_set_count = 0;
        if self.use_upper { char_set_count += 1; }
        if self.use_lower { char_set_count += 1; }
        if self.use_digits { char_set_count += 1; }
        if self.use_special { char_set_count += 1; }
        
        if char_set_count == 0 {
            return Err("At least one character set must be selected".to_string());
        }
        
        Ok(())
    }

    fn build_character_set(&self) -> String {
        let mut charset = String::new();
        if self.use_upper { charset.push_str(UPPER); }
        if self.use_lower { charset.push_str(LOWER); }
        if self.use_digits { charset.push_str(DIGITS); }
        if self.use_special { charset.push_str(SPECIAL); }
        charset
    }
}

fn generate_password(config: &PasswordConfig) -> Result<String, String> {
    config.validate()?;
    
    let charset = config.build_character_set();
    let charset_bytes = charset.as_bytes();
    let charset_len = charset_bytes.len();
    
    if charset_len == 0 {
        return Err("Character set is empty".to_string());
    }
    
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(config.length);
    
    for _ in 0..config.length {
        let idx = rng.gen_range(0..charset_len);
        password.push(charset_bytes[idx] as char);
    }
    
    Ok(password)
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("Secure Password Generator");
    println!("=========================");
    
    let mut config = PasswordConfig::new();
    
    let length_input = get_user_input("Password length (default: 16): ");
    if !length_input.is_empty() {
        if let Ok(length) = length_input.parse::<usize>() {
            config.length = length;
        }
    }
    
    let upper_input = get_user_input("Include uppercase letters? (Y/n): ");
    if !upper_input.is_empty() {
        config.use_upper = parse_bool_input(&upper_input);
    }
    
    let lower_input = get_user_input("Include lowercase letters? (Y/n): ");
    if !lower_input.is_empty() {
        config.use_lower = parse_bool_input(&lower_input);
    }
    
    let digits_input = get_user_input("Include digits? (Y/n): ");
    if !digits_input.is_empty() {
        config.use_digits = parse_bool_input(&digits_input);
    }
    
    let special_input = get_user_input("Include special characters? (Y/n): ");
    if !special_input.is_empty() {
        config.use_special = parse_bool_input(&special_input);
    }
    
    match generate_password(&config) {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password length: {} characters", password.len());
        }
        Err(e) => {
            eprintln!("Error generating password: {}", e);
        }
    }
}