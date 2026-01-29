use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_valid_numbers(numbers: Vec<Option<f64>>) -> Vec<f64> {
    numbers
        .into_iter()
        .filter_map(|num| num)
        .filter(|&n| n.is_finite() && n > 0.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 1, 4];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_filter_valid_numbers() {
        let input = vec![Some(1.5), None, Some(-2.0), Some(f64::INFINITY), Some(3.0)];
        let result = filter_valid_numbers(input);
        assert_eq!(result, vec![1.5, 3.0]);
    }
}