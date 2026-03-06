use rand::Rng;
use std::io;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

fn main() {
    println!("Secure Password Generator");
    println!("==========================");
    
    let length = get_password_length();
    let char_sets = select_character_sets();
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    println!("Password Strength: {}", evaluate_strength(&password));
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (8-64):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 64 => return length,
            Ok(_) => println!("Length must be between 8 and 64"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn select_character_sets() -> Vec<String> {
    let mut char_sets = Vec::new();
    let mut selected = 0;
    
    println!("\nSelect character sets to include:");
    println!("1. Uppercase letters");
    println!("2. Lowercase letters");
    println!("3. Numbers");
    println!("4. Symbols");
    
    loop {
        println!("\nEnter selection (1-4, 'd' when done, at least 2 sets required):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim() {
            "1" => {
                if !char_sets.contains(&UPPERCASE.to_string()) {
                    char_sets.push(UPPERCASE.to_string());
                    selected += 1;
                    println!("✓ Uppercase letters added");
                }
            }
            "2" => {
                if !char_sets.contains(&LOWERCASE.to_string()) {
                    char_sets.push(LOWERCASE.to_string());
                    selected += 1;
                    println!("✓ Lowercase letters added");
                }
            }
            "3" => {
                if !char_sets.contains(&NUMBERS.to_string()) {
                    char_sets.push(NUMBERS.to_string());
                    selected += 1;
                    println!("✓ Numbers added");
                }
            }
            "4" => {
                if !char_sets.contains(&SYMBOLS.to_string()) {
                    char_sets.push(SYMBOLS.to_string());
                    selected += 1;
                    println!("✓ Symbols added");
                }
            }
            "d" => {
                if selected >= 2 {
                    break;
                } else {
                    println!("Please select at least 2 character sets");
                }
            }
            _ => println!("Invalid selection"),
        }
    }
    
    char_sets
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::with_capacity(length);
    
    // Ensure at least one character from each selected set
    for char_set in char_sets {
        let idx = rng.gen_range(0..char_set.len());
        password.push(char_set.chars().nth(idx).unwrap());
    }
    
    // Fill remaining length with random characters from all selected sets
    let all_chars: String = char_sets.concat();
    while password.len() < length {
        let idx = rng.gen_range(0..all_chars.len());
        password.push(all_chars.chars().nth(idx).unwrap());
    }
    
    // Shuffle the password to randomize position of required characters
    let mut chars: Vec<char> = password.chars().collect();
    for i in 0..chars.len() {
        let j = rng.gen_range(0..chars.len());
        chars.swap(i, j);
    }
    
    chars.into_iter().collect()
}

fn evaluate_strength(password: &str) -> String {
    let length = password.len();
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_symbol = false;
    
    for ch in password.chars() {
        if ch.is_ascii_uppercase() {
            has_upper = true;
        } else if ch.is_ascii_lowercase() {
            has_lower = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if ch.is_ascii_punctuation() {
            has_symbol = true;
        }
    }
    
    let complexity = [has_upper, has_lower, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();
    
    match (length, complexity) {
        (len, _) if len < 8 => "Very Weak",
        (8..=11, 2) => "Weak",
        (8..=11, 3..=4) => "Moderate",
        (12..=15, 2) => "Moderate",
        (12..=15, 3..=4) => "Strong",
        (16.., 2) => "Strong",
        (16.., 3..=4) => "Very Strong",
        _ => "Weak",
    }.to_string()
}