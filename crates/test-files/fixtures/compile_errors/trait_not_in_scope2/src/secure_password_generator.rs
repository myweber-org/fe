use rand::Rng;
use std::io;

const DEFAULT_LENGTH: usize = 16;

fn main() {
    println!("Secure Password Generator");
    println!("=========================");

    let length = get_password_length();
    let include_uppercase = get_yes_no_input("Include uppercase letters? (y/n): ");
    let include_lowercase = get_yes_no_input("Include lowercase letters? (y/n): ");
    let include_numbers = get_yes_no_input("Include numbers? (y/n): ");
    let include_symbols = get_yes_no_input("Include symbols? (y/n): ");

    let password = generate_password(
        length,
        include_uppercase,
        include_lowercase,
        include_numbers,
        include_symbols,
    );

    match password {
        Some(pwd) => println!("\nGenerated Password: {}", pwd),
        None => println!("\nError: No character sets selected!"),
    }
}

fn get_password_length() -> usize {
    loop {
        println!("\nEnter password length (default: {}): ", DEFAULT_LENGTH);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return DEFAULT_LENGTH;
        }

        match trimmed.parse::<usize>() {
            Ok(length) if length >= 4 && length <= 128 => return length,
            Ok(_) => println!("Password length must be between 4 and 128 characters"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn get_yes_no_input(prompt: &str) -> bool {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}

fn generate_password(
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) -> Option<String> {
    let mut character_set = String::new();
    
    if uppercase {
        character_set.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if lowercase {
        character_set.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if numbers {
        character_set.push_str("0123456789");
    }
    if symbols {
        character_set.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    if character_set.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    let chars: Vec<char> = character_set.chars().collect();
    
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx]
        })
        .collect();

    Some(password)
}