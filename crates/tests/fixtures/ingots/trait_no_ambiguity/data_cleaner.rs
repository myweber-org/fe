use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn remove_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let normalized = normalize_strings(strings);
    let non_empty = remove_empty_strings(normalized);
    deduplicate(non_empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(deduplicate(input), expected);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let expected = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(normalize_strings(input), expected);
    }

    #[test]
    fn test_remove_empty_strings() {
        let input = vec!["a".to_string(), "".to_string(), "b".to_string()];
        let expected = vec!["a".to_string(), "b".to_string()];
        assert_eq!(remove_empty_strings(input), expected);
    }

    #[test]
    fn test_clean_string_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "Banana  ".to_string(),
            "banana".to_string(),
        ];
        let expected = vec!["apple".to_string(), "banana".to_string()];
        assert_eq!(clean_string_data(input), expected);
    }
}