
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut unique_lines: HashSet<&str> = HashSet::new();
    
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            unique_lines.insert(trimmed);
        }
    }
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    
    for line in stdin.lock().lines() {
        input.push_str(&line?);
        input.push('\n');
    }
    
    let cleaned = clean_data(&input);
    io::stdout().write_all(cleaned.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "apple\nbanana\napple\ncherry\nbanana\n";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_clean_data_with_empty_lines() {
        let input = "apple\n\nbanana\n\napple\n";
        let expected = "apple\nbanana";
        assert_eq!(clean_data(input), expected);
    }
}