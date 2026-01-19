use rand::Rng;
use std::io;

fn main() {
    let length = get_password_length();
    let use_uppercase = get_yes_no_input("Include uppercase letters? (y/n): ");
    let use_lowercase = get_yes_no_input("Include lowercase letters? (y/n): ");
    let use_digits = get_yes_no_input("Include digits? (y/n): ");
    let use_special = get_yes_no_input("Include special characters? (y/n): ");

    let password = generate_password(length, use_uppercase, use_lowercase, use_digits, use_special);
    println!("Generated password: {}", password);
}

fn get_password_length() -> usize {
    loop {
        println!("Enter password length (8-128): ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        match input.trim().parse::<usize>() {
            Ok(length) if length >= 8 && length <= 128 => return length,
            Ok(_) => println!("Length must be between 8 and 128"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn get_yes_no_input(prompt: &str) -> bool {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        
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
    digits: bool,
    special: bool,
) -> String {
    let mut rng = rand::thread_rng();
    let mut charset = String::new();
    
    if uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if lowercase {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if digits {
        charset.push_str("0123456789");
    }
    if special {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    if charset.is_empty() {
        charset = String::from("abcdefghijklmnopqrstuvwxyz");
    }
    
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}