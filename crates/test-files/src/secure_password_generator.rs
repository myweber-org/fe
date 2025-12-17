use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected");
        return;
    }
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("Enter password length (default: {}): ", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        
        let input = input.trim();
        
        if input.is_empty() {
            return DEFAULT_LENGTH;
        }
        
        match input.parse::<usize>() {
            Ok(length) if length >= 4 && length <= 128 => return length,
            Ok(_) => println!("Length must be between 4 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut rng = rand::thread_rng();
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Digits (0-9)");
    println!("4. Symbols (!@#$% etc.)");
    println!("Press Enter to use default (all sets)");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    
    let input = input.trim();
    
    if input.is_empty() {
        return vec![
            UPPERCASE.to_string(),
            LOWERCASE.to_string(),
            DIGITS.to_string(),
            SYMBOLS.to_string(),
        ];
    }
    
    for ch in input.chars() {
        match ch {
            '1' => char_sets.push(UPPERCASE.to_string()),
            '2' => char_sets.push(LOWERCASE.to_string()),
            '3' => char_sets.push(DIGITS.to_string()),
            '4' => char_sets.push(SYMBOLS.to_string()),
            _ => continue,
        }
    }
    
    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    let all_chars: String = char_sets.concat();
    
    for _ in 0..length {
        let idx = rng.gen_range(0..all_chars.len());
        password.push(all_chars.chars().nth(idx).unwrap());
    }
    
    password
}

fn evaluate_strength(password: &str) -> String {
    let length = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_ascii_alphanumeric());
    
    let mut score = 0;
    
    if length >= 12 { score += 2; }
    else if length >= 8 { score += 1; }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_symbol { score += 1; }
    
    match score {
        0..=2 => "Weak",
        3..=4 => "Medium",
        5..=6 => "Strong",
        _ => "Very Strong",
    }.to_string()
}