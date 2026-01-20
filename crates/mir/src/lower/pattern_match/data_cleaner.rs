
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

    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_and_deduplicate() {
        let input = vec![
            "Apple".to_string(),
            "  apple ".to_string(),
            "Banana".to_string(),
            "banana".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = normalize_and_deduplicate_strings(input);
        assert_eq!(result, vec!["Apple".to_string(), "Banana".to_string(), "Cherry".to_string()]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<String> = vec![];
        let result = normalize_and_deduplicate_strings(input);
        assert!(result.is_empty());
    }
}