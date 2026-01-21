
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
    let char_sets = select_character_sets();
    
    let password = generate_password(length, &char_sets);
    println!("\nGenerated Password: {}", password);
    
    let entropy = calculate_entropy(length, char_sets.len());
    println!("Estimated Entropy: {:.2} bits", entropy);
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

fn select_character_sets() -> Vec<String> {
    let mut selected = Vec::new();
    let options = [
        ("Uppercase letters", UPPERCASE.to_string()),
        ("Lowercase letters", LOWERCASE.to_string()),
        ("Digits", DIGITS.to_string()),
        ("Symbols", SYMBOLS.to_string()),
    ];
    
    println!("\nSelect character sets to include:");
    for (i, (name, _)) in options.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }
    println!("5. All character sets");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
        
    let choices: Vec<usize> = input
        .trim()
        .split(',')
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .collect();
        
    if choices.contains(&5) {
        return options.iter().map(|(_, chars)| chars.clone()).collect();
    }
    
    for choice in choices {
        if choice >= 1 && choice <= 4 {
            selected.push(options[choice - 1].1.clone());
        }
    }
    
    if selected.is_empty() {
        println!("No valid selections, using all character sets");
        options.iter().map(|(_, chars)| chars.clone()).collect()
    } else {
        selected
    }
}

fn generate_password(length: usize, char_sets: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let all_chars: String = char_sets.concat();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..all_chars.len());
            all_chars.chars().nth(idx).unwrap()
        })
        .collect()
}

fn calculate_entropy(length: usize, char_set_count: usize) -> f64 {
    let charset_size: f64 = match char_set_count {
        1 => 26.0,    // Only letters
        2 => 52.0,    // Letters (upper + lower)
        3 => 62.0,    // Letters + digits
        4 => 94.0,    // All printable ASCII
        _ => 94.0,
    };
    
    (length as f64) * charset_size.log2()
}