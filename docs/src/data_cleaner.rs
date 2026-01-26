use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn deduplicate(&self) -> Vec<T> {
        let mut seen = HashSet::new();
        self.data
            .iter()
            .filter(|item| seen.insert(*item))
            .cloned()
            .collect()
    }

    pub fn normalize<F>(&self, transform: F) -> Vec<T>
    where
        F: Fn(&T) -> T,
    {
        self.data.iter().map(transform).collect()
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        self.data.iter().filter(|&x| predicate(x)).cloned().collect()
    }

    pub fn get_stats(&self) -> (usize, usize) {
        let total = self.data.len();
        let unique = self.deduplicate().len();
        (total, unique)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let cleaner = DataCleaner::new(vec![1, 2, 2, 3, 3, 3]);
        let deduped = cleaner.deduplicate();
        assert_eq!(deduped, vec![1, 2, 3]);
    }

    #[test]
    fn test_normalize() {
        let cleaner = DataCleaner::new(vec![1, 2, 3]);
        let normalized = cleaner.normalize(|&x| x * 2);
        assert_eq!(normalized, vec![2, 4, 6]);
    }

    #[test]
    fn test_filter() {
        let cleaner = DataCleaner::new(vec![1, 2, 3, 4, 5]);
        let filtered = cleaner.filter(|&x| x % 2 == 0);
        assert_eq!(filtered, vec![2, 4]);
    }

    #[test]
    fn test_stats() {
        let cleaner = DataCleaner::new(vec![1, 2, 2, 3, 3, 3]);
        let (total, unique) = cleaner.get_stats();
        assert_eq!(total, 6);
        assert_eq!(unique, 3);
    }
}