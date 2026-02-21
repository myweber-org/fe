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

fn generate_password(config: &PasswordConfig) -> Result<String, String> {
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
        return Err("At least one character set must be selected".to_string());
    }
    
    if config.length == 0 {
        return Err("Password length must be greater than 0".to_string());
    }
    
    let mut rng = rand::thread_rng();
    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..character_pool.len());
            character_pool.chars().nth(idx).unwrap()
        })
        .collect();
    
    Ok(password)
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
    println!("Secure Password Generator");
    println!("=========================");
    
    let length_input = get_user_input(&format!("Password length (default: {}): ", DEFAULT_LENGTH));
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };
    
    let use_uppercase = parse_bool_input(&get_user_input("Include uppercase letters? (Y/n): "));
    let use_lowercase = parse_bool_input(&get_user_input("Include lowercase letters? (Y/n): "));
    let use_numbers = parse_bool_input(&get_user_input("Include numbers? (Y/n): "));
    let use_symbols = parse_bool_input(&get_user_input("Include symbols? (Y/n): "));
    
    let config = PasswordConfig {
        length,
        use_uppercase,
        use_lowercase,
        use_numbers,
        use_symbols,
    };
    
    match generate_password(&config) {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Strength: {}", evaluate_password_strength(&password));
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn evaluate_password_strength(password: &str) -> &'static str {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    
    let mut score = 0;
    if length >= 8 { score += 1; }
    if length >= 12 { score += 1; }
    if length >= 16 { score += 1; }
    if has_upper && has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Medium",
        5..=6 => "Strong",
        _ => "Very Strong",
    }
}