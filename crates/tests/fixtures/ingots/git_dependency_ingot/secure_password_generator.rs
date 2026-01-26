use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    fn new() -> Self {
        PasswordGenerator {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    fn with_numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    fn with_symbols(mut self, enable: bool) -> Self {
        self.use_symbols = enable;
        self
    }

    fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        let mut character_pool = String::new();
        
        if self.use_uppercase {
            character_pool.push_str(UPPERCASE);
        }
        if self.use_lowercase {
            character_pool.push_str(LOWERCASE);
        }
        if self.use_numbers {
            character_pool.push_str(NUMBERS);
        }
        if self.use_symbols {
            character_pool.push_str(SYMBOLS);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool.chars().nth(idx).unwrap()
            })
            .collect();

        Ok(password)
    }
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
    
    let length_input = get_user_input("Enter password length (default: 16):");
    let length = if length_input.is_empty() {
        DEFAULT_LENGTH
    } else {
        length_input.parse().unwrap_or(DEFAULT_LENGTH)
    };

    let use_uppercase = parse_bool_input(&get_user_input("Include uppercase letters? (Y/n):"));
    let use_lowercase = parse_bool_input(&get_user_input("Include lowercase letters? (Y/n):"));
    let use_numbers = parse_bool_input(&get_user_input("Include numbers? (Y/n):"));
    let use_symbols = parse_bool_input(&get_user_input("Include symbols? (Y/n):"));

    let generator = PasswordGenerator::new()
        .with_length(length)
        .with_uppercase(use_uppercase)
        .with_lowercase(use_lowercase)
        .with_numbers(use_numbers)
        .with_symbols(use_symbols);

    match generator.generate() {
        Ok(password) => {
            println!("\nGenerated Password: {}", password);
            println!("Password Length: {}", password.len());
            
            let mut strength = "Weak";
            if password.len() >= 12 && use_uppercase && use_lowercase && use_numbers && use_symbols {
                strength = "Strong";
            } else if password.len() >= 8 && (use_uppercase || use_lowercase) && use_numbers {
                strength = "Medium";
            }
            
            println!("Estimated Strength: {}", strength);
        }
        Err(e) => println!("Error: {}", e),
    }
}