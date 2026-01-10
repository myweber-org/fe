use std::collections::HashSet;
use std::io::{self, BufRead};

fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

fn read_input() -> Vec<String> {
    let stdin = io::stdin();
    let mut lines = Vec::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(content) => {
                if content.trim().is_empty() {
                    break;
                }
                lines.push(content.trim().to_string());
            }
            Err(_) => break,
        }
    }
    lines
}

fn main() {
    let input_data = read_input();
    let cleaned_data = clean_data(input_data);
    for item in cleaned_data {
        println!("{}", item);
    }
}use regex::Regex;

pub fn clean_string(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_string("Test@#$%^&*()String"), "TestString");
        assert_eq!(clean_string(""), "");
        assert_eq!(clean_string("123_456"), "123456");
    }
}