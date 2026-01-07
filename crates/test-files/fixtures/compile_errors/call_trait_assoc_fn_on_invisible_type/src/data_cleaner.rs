use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) {
        let unique_set: HashSet<String> = self.data.drain(..).collect();
        self.data = unique_set.into_iter().collect();
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.data {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.data
    }

    pub fn process(&mut self) {
        self.normalize_strings();
        self.remove_duplicates();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut cleaner = DataCleaner::new(vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "banana ".to_string(),
            "Cherry".to_string(),
        ]);

        cleaner.process();
        let result = cleaner.get_data();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }
}