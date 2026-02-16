use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    data.iter()
        .filter(|item| {
            let is_unique = seen.insert(*item);
            is_unique
        })
        .cloned()
        .collect()
}

pub fn remove_nulls<T: Default + PartialEq>(data: &[Option<T>]) -> Vec<T> {
    data.iter()
        .filter_map(|item| {
            if let Some(value) = item {
                if *value != T::default() {
                    return Some(value.clone());
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_removes_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let cleaned = clean_data(&data);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_remove_nulls_filters_none_and_default() {
        let data = vec![Some(1), None, Some(0), Some(2), Some(0)];
        let cleaned = remove_nulls(&data);
        assert_eq!(cleaned, vec![1, 2]);
    }
}