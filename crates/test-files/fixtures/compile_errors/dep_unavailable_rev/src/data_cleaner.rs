use std::collections::HashSet;

pub fn clean_and_sort_data<T: Ord + Clone>(data: &[T]) -> Vec<T> {
    let mut unique_items: HashSet<_> = data.iter().cloned().collect();
    let mut result: Vec<T> = unique_items.drain().collect();
    result.sort();
    result
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
    fn test_clean_and_sort_strings() {
        let input = vec!["banana", "apple", "cherry", "apple", "banana"];
        let result = clean_and_sort_data(&input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}