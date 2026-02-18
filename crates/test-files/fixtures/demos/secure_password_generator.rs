use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");

    let length = get_password_length();
    let char_sets = select_character_sets();
    
    if char_sets.is_empty() {
        println!("Error: At least one character set must be selected!");
        return;
    }

    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    print_strength_indicator(&password);
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-128):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 128 => return length,
            Ok(_) => println!("Length must be between 8 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut rng = rand::thread_rng();
    
    println!("\nSelect character sets (enter numbers separated by spaces):");
    println!("1. Uppercase letters (A-Z)");
    println!("2. Lowercase letters (a-z)");
    println!("3. Digits (0-9)");
    println!("4. Symbols (!@#$% etc.)");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    for num_str in input.split_whitespace() {
        match num_str {
            "1" => char_sets.push(UPPERCASE.to_string()),
            "2" => char_sets.push(LOWERCASE.to_string()),
            "3" => char_sets.push(DIGITS.to_string()),
            "4" => char_sets.push(SYMBOLS.to_string()),
            _ => println!("Ignoring invalid option: {}", num_str),
        }
    }
    
    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    for _ in 0..length {
        let set_index = rng.gen_range(0..char_sets.len());
        let charset = &char_sets[set_index];
        let char_index = rng.gen_range(0..charset.len());
        
        password.push(charset.chars().nth(char_index).unwrap());
    }
    
    password
}

fn print_strength_indicator(password: &str) {
    let mut score = 0;
    
    if password.chars().any(|c| UPPERCASE.contains(c)) {
        score += 1;
    }
    if password.chars().any(|c| LOWERCASE.contains(c)) {
        score += 1;
    }
    if password.chars().any(|c| DIGITS.contains(c)) {
        score += 1;
    }
    if password.chars().any(|c| SYMBOLS.contains(c)) {
        score += 1;
    }
    
    let length_score = password.len() / 8;
    score += length_score.min(3);
    
    println!("\nPassword Strength: {}", match score {
        0..=2 => "Weak",
        3..=4 => "Moderate",
        5..=6 => "Strong",
        _ => "Very Strong",
    });
}