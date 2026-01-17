use regex::Regex;
use std::collections::HashSet;

pub fn sanitize_string(input: &str) -> String {
    let trimmed = input.trim();
    let re = Regex::new(r"\s+").unwrap();
    let normalized_whitespace = re.replace_all(trimmed, " ");
    
    normalized_whitespace.to_string()
}

pub fn remove_duplicate_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for word in words {
        if seen.insert(word) {
            result.push(word);
        }
    }
    
    result.join(" ")
}

pub fn normalize_case(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  hello    world  "), "hello world");
        assert_eq!(sanitize_string("data\n\tprocessing"), "data processing");
    }

    #[test]
    fn test_remove_duplicate_words() {
        assert_eq!(remove_duplicate_words("hello hello world world"), "hello world");
        assert_eq!(remove_duplicate_words("a b a c b d"), "a b c d");
    }

    #[test]
    fn test_normalize_case() {
        assert_eq!(normalize_case("hELLO"), "Hello");
        assert_eq!(normalize_case(""), "");
    }
}