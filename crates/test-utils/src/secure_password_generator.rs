use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordConfig {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_symbols: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_symbols: true,
        }
    }
}

fn generate_password(config: &PasswordConfig) -> String {
    let mut charset = String::new();
    
    if config.use_uppercase {
        charset.push_str(UPPERCASE);
    }
    if config.use_lowercase {
        charset.push_str(LOWERCASE);
    }
    if config.use_digits {
        charset.push_str(DIGITS);
    }
    if config.use_symbols {
        charset.push_str(SYMBOLS);
    }
    
    if charset.is_empty() {
        return "Error: No character set selected".to_string();
    }
    
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    let mut rng = rand::thread_rng();
    
    (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn parse_bool_input(input: &str) -> bool {
    match input.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => true,
        _ => false,
    }
}

fn main() {
    println!("=== Secure Password Generator ===");
    
    let length_input = get_user_input(&format!("Password length (default: {}):", DEFAULT_LENGTH));
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };
    
    let config = PasswordConfig {
        length,
        use_uppercase: parse_bool_input(&get_user_input("Include uppercase letters? (Y/n):")),
        use_lowercase: parse_bool_input(&get_user_input("Include lowercase letters? (Y/n):")),
        use_digits: parse_bool_input(&get_user_input("Include digits? (Y/n):")),
        use_symbols: parse_bool_input(&get_user_input("Include symbols? (Y/n):")),
    };
    
    let password = generate_password(&config);
    println!("\nGenerated Password: {}", password);
    println!("Password length: {} characters", password.len());
}