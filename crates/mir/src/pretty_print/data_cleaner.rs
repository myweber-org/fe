use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    
    println!("Enter data (press Ctrl+D on Unix or Ctrl+Z on Windows to finish):");
    for line in stdin.lock().lines() {
        input.push_str(&line?);
        input.push('\n');
    }
    
    let cleaned = clean_data(&input);
    println!("\nCleaned data:");
    io::stdout().write_all(cleaned.as_bytes())?;
    
    Ok(())
}