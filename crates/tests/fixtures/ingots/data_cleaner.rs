
use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    data.into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_removes_duplicates() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let result = clean_data(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_clean_data_preserves_order() {
        let input = vec!["apple", "banana", "apple", "cherry"];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_clean_data_empty_input() {
        let input: Vec<i32> = vec![];
        let result = clean_data(input);
        assert!(result.is_empty());
    }
}