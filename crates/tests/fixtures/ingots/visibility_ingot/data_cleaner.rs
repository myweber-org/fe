
use regex::Regex;

pub fn clean_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_alphanumeric() {
        assert_eq!(clean_alphanumeric("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_alphanumeric("Test@#$%^&*()String"), "TestString");
        assert_eq!(clean_alphanumeric("123_456-789"), "123456789");
        assert_eq!(clean_alphanumeric(""), "");
    }
}
use std::collections::HashSet;

pub fn normalize_and_deduplicate_strings(strings: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for s in strings {
        let normalized = s.trim().to_lowercase();
        if seen.insert(normalized.clone()) {
            result.push(s);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_and_deduplicate() {
        let input = vec![
            "  Apple".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana ".to_string(),
            "Cherry".to_string(),
        ];
        let result = normalize_and_deduplicate_strings(input);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "  Apple");
        assert_eq!(result[1], "BANANA");
        assert_eq!(result[2], "Cherry");
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<String> = vec![];
        let result = normalize_and_deduplicate_strings(input);
        assert!(result.is_empty());
    }
}