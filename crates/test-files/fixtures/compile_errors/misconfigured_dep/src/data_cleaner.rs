use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Option<T>>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(data: Vec<Option<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_nulls(&self) -> Vec<T> {
        self.data
            .iter()
            .filter_map(|item| item.clone())
            .collect()
    }

    pub fn remove_duplicates(&self) -> Vec<T> {
        let mut seen = HashSet::new();
        self.data
            .iter()
            .filter_map(|item| item.clone())
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn clean(&self) -> Vec<T> {
        let without_nulls = self.remove_nulls();
        let mut seen = HashSet::new();
        without_nulls
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_nulls() {
        let data = vec![Some(1), None, Some(2), None, Some(3)];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.remove_nulls();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_remove_duplicates() {
        let data = vec![Some(1), Some(2), Some(1), Some(3), Some(2)];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.remove_duplicates();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    #[test]
    fn test_clean() {
        let data = vec![Some(1), None, Some(2), Some(1), None, Some(3)];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.clean();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }
}