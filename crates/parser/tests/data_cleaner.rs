use std::collections::HashSet;

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in data {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_by_length(strings: Vec<String>, min_len: usize) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| s.len() >= min_len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let result = remove_duplicates(&data);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  TEST  ".to_string(),
        ];
        let result = normalize_strings(&strings);
        assert_eq!(result, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_filter_by_length() {
        let strings = vec![
            "a".to_string(),
            "ab".to_string(),
            "abc".to_string(),
            "abcd".to_string(),
        ];
        let result = filter_by_length(strings, 3);
        assert_eq!(result, vec!["abc", "abcd"]);
    }
}