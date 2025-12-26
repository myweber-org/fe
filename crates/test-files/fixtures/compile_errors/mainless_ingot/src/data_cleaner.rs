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

    pub fn deduplicate(&self) -> Vec<T> {
        let mut seen = HashSet::new();
        self.remove_nulls()
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn clean(&self) -> Vec<T> {
        self.deduplicate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_nulls() {
        let data = vec![Some(1), None, Some(2), Some(1), None];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.remove_nulls();
        assert_eq!(result, vec![1, 2, 1]);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![Some("a"), None, Some("b"), Some("a"), Some("c")];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.deduplicate();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_clean() {
        let data = vec![Some(10), None, Some(20), Some(10), None, Some(30)];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.clean();
        assert_eq!(result, vec![10, 20, 30]);
    }
}