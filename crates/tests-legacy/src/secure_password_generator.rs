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
}use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let mut character_pool = String::new();
    
    if config.use_uppercase {
        character_pool.push_str(UPPERCASE);
    }
    if config.use_lowercase {
        character_pool.push_str(LOWERCASE);
    }
    if config.use_numbers {
        character_pool.push_str(NUMBERS);
    }
    if config.use_symbols {
        character_pool.push_str(SYMBOLS);
    }
    
    if character_pool.is_empty() {
        return String::from("Error: No character types selected");
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..character_pool.len());
            character_pool.chars().nth(idx).unwrap()
        })
        .collect();
    
    password
}

fn get_user_input() -> PasswordConfig {
    let mut config = PasswordConfig::default();
    
    println!("Password Generator");
    println!("==================");
    
    println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if let Ok(length) = input.trim().parse::<usize>() {
        if length >= 4 && length <= 128 {
            config.length = length;
        }
    }
    
    println!("Include uppercase letters? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    config.use_uppercase = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include lowercase letters? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    config.use_lowercase = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include numbers? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    config.use_numbers = !input.trim().eq_ignore_ascii_case("n");
    
    println!("Include symbols? (y/n, default: y): ");
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    config.use_symbols = !input.trim().eq_ignore_ascii_case("n");
    
    config
}

fn main() {
    let config = get_user_input();
    let password = generate_password(&config);
    
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
    
    let mut strength = "Weak";
    if password.len() >= 12 && config.use_uppercase && config.use_lowercase && config.use_numbers && config.use_symbols {
        strength = "Strong";
    } else if password.len() >= 8 {
        strength = "Medium";
    }
    
    println!("Estimated Strength: {}", strength);
}