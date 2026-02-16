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
}use std::collections::HashSet;

pub fn remove_duplicates<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in input {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates_integers() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_remove_duplicates_strings() {
        let input = vec!["apple", "banana", "apple", "orange", "banana"];
        let expected = vec!["apple", "banana", "orange"];
        assert_eq!(remove_duplicates(&input), expected);
    }

    #[test]
    fn test_remove_duplicates_empty() {
        let input: Vec<i32> = vec![];
        let expected: Vec<i32> = vec![];
        assert_eq!(remove_duplicates(&input), expected);
    }
}