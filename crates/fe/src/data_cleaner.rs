use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
    normalized: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
            normalized: Vec::new(),
        }
    }

    pub fn process(&mut self, item: T) -> bool {
        if self.seen.insert(item.clone()) {
            self.normalized.push(item);
            true
        } else {
            false
        }
    }

    pub fn get_normalized(&self) -> &[T] {
        &self.normalized
    }

    pub fn clear(&mut self) {
        self.seen.clear();
        self.normalized.clear();
    }

    pub fn process_batch<I>(&mut self, items: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let initial_len = self.normalized.len();
        for item in items {
            self.process(item);
        }
        self.normalized.len() - initial_len
    }
}

impl<T> Default for DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.process("apple".to_string()));
        assert!(!cleaner.process("apple".to_string()));
        assert!(cleaner.process("banana".to_string()));
        assert_eq!(cleaner.get_normalized().len(), 2);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let items = vec![1, 2, 2, 3, 1, 4];
        let added = cleaner.process_batch(items);
        assert_eq!(added, 4);
        assert_eq!(cleaner.get_normalized(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_clear() {
        let mut cleaner = DataCleaner::new();
        cleaner.process("test".to_string());
        cleaner.clear();
        assert!(cleaner.get_normalized().is_empty());
        assert!(cleaner.process("test".to_string()));
    }
}