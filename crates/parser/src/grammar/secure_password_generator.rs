use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");
    
    let length = get_password_length();
    let char_set = select_character_sets();
    
    let password = generate_password(length, &char_set);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (default: {}):", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
            
        let input = input.trim();
        if input.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match input.parse::<usize>() {
            Ok(length) if length >= 8 && length <= 128 => return length,
            Ok(_) => println!("Length must be between 8 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> String {
    let mut char_set = String::new();
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Digits (0-9)");
    println!("4. Symbols (!@#$% etc.)");
    println!("Enter selections (e.g., '1234' for all):");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
        
    let selections = input.trim();
    
    if selections.contains('1') || selections.is_empty() {
        char_set.push_str(UPPERCASE);
    }
    if selections.contains('2') || selections.is_empty() {
        char_set.push_str(LOWERCASE);
    }
    if selections.contains('3') || selections.is_empty() {
        char_set.push_str(DIGITS);
    }
    if selections.contains('4') || selections.is_empty() {
        char_set.push_str(SYMBOLS);
    }
    
    if char_set.is_empty() {
        char_set = format!("{}{}{}{}", UPPERCASE, LOWERCASE, DIGITS, SYMBOLS);
    }
    
    char_set
}

fn generate_password(length: usize, char_set: &str) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = char_set.chars().collect();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx]
        })
        .collect()
}

fn evaluate_strength(password: &str) -> String {
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());
    
    let mut score = 0;
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }
    
    let length = password.len();
    if length >= 20 { score += 2; }
    else if length >= 12 { score += 1; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Medium",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}