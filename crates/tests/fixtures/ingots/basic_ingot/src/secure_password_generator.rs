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

fn get_user_input() -> PasswordConfig {
    let mut config = PasswordConfig::default();
    
    println!("Password Generator Configuration");
    println!("--------------------------------");
    
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
    if password.len() >= 12 {
        strength = "Strong";
    } else if password.len() >= 8 {
        strength = "Medium";
    }
    
    println!("Strength: {}", strength);
}