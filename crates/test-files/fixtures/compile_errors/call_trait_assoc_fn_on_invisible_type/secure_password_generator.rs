use rand::Rng;
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
        return String::from("Error: No character sets selected");
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

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
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
    
    let mut config = PasswordConfig {
        length,
        ..Default::default()
    };
    
    config.use_uppercase = parse_bool_input(&get_user_input("Include uppercase letters? (Y/n):"));
    config.use_lowercase = parse_bool_input(&get_user_input("Include lowercase letters? (Y/n):"));
    config.use_numbers = parse_bool_input(&get_user_input("Include numbers? (Y/n):"));
    config.use_symbols = parse_bool_input(&get_user_input("Include symbols? (Y/n):"));
    
    let password = generate_password(&config);
    println!("\nGenerated Password: {}", password);
    println!("Password Length: {}", password.len());
    
    if config.use_uppercase && password.chars().any(|c| c.is_ascii_uppercase()) {
        println!("✓ Contains uppercase letters");
    }
    if config.use_lowercase && password.chars().any(|c| c.is_ascii_lowercase()) {
        println!("✓ Contains lowercase letters");
    }
    if config.use_numbers && password.chars().any(|c| c.is_ascii_digit()) {
        println!("✓ Contains numbers");
    }
    if config.use_symbols && password.chars().any(|c| SYMBOLS.contains(c)) {
        println!("✓ Contains symbols");
    }
}