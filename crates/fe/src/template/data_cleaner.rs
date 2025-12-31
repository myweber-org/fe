use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize(&mut self) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.data = self
            .data
            .iter()
            .map(|s| {
                let normalized = s
                    .as_ref()
                    .trim()
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>();
                normalized.into()
            })
            .collect();
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(predicate);
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn process_string_data(raw_strings: Vec<&str>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(raw_strings.into_iter().map(String::from).collect());
    cleaner
        .deduplicate()
        .normalize()
        .filter(|s| !s.is_empty())
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec!["apple", "banana", "apple", "orange", "banana"];
        let mut cleaner = DataCleaner::new(data.into_iter().map(String::from).collect());
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data().len(), 3);
    }

    #[test]
    fn test_normalization() {
        let data = vec!["  Apple ", "BANANA", "OrAnGe!!"];
        let mut cleaner = DataCleaner::new(data.into_iter().map(String::from).collect());
        cleaner.normalize();
        let result = cleaner.get_data();
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"orange".to_string()));
    }

    #[test]
    fn test_empty_filter() {
        let data = vec!["valid", "", "   ", "another"];
        let result = process_string_data(data);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"valid".to_string()));
        assert!(result.contains(&"another".to_string()));
    }
}