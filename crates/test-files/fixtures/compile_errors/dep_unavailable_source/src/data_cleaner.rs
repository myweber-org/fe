use std::collections::HashSet;

pub fn clean_and_sort_data<T: Ord + Clone + std::hash::Hash>(data: &[T]) -> Vec<T> {
    let unique_items: HashSet<_> = data.iter().cloned().collect();
    let mut sorted_unique: Vec<_> = unique_items.into_iter().collect();
    sorted_unique.sort();
    sorted_unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort() {
        let input = vec![5, 2, 8, 2, 5, 1, 9];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<i32> = vec![];
        let result = clean_and_sort_data(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_string_data() {
        let input = vec!["banana", "apple", "banana", "cherry"];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}