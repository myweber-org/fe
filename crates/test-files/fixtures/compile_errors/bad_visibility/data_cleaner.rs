use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

fn read_input() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let mut lines = Vec::new();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }
        lines.push(line.trim().to_string());
    }
    
    Ok(lines)
}

fn write_output(cleaned_data: &[String]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    for item in cleaned_data {
        writeln!(handle, "{}", item)?;
    }
    
    Ok(())
}

fn main() -> io::Result<()> {
    let input_data = read_input()?;
    let cleaned_data = clean_data(input_data);
    write_output(&cleaned_data)?;
    Ok(())
}