use std::collections::HashMap;

pub struct DataCleaner {
    pub column_defaults: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            column_defaults: HashMap::new(),
        }
    }

    pub fn set_default(&mut self, column: &str, default_value: &str) {
        self.column_defaults.insert(column.to_string(), default_value.to_string());
    }

    pub fn clean_row(&self, row: &mut HashMap<String, String>) {
        for (column, default_value) in &self.column_defaults {
            if !row.contains_key(column) || row.get(column).unwrap().trim().is_empty() {
                row.insert(column.clone(), default_value.clone());
            }
        }

        for (_, value) in row.iter_mut() {
            *value = value.trim().to_lowercase();
        }
    }

    pub fn clean_dataset(&self, dataset: &mut Vec<HashMap<String, String>>) {
        for row in dataset {
            self.clean_row(row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_row() {
        let mut cleaner = DataCleaner::new();
        cleaner.set_default("name", "unknown");
        cleaner.set_default("age", "0");

        let mut row = HashMap::new();
        row.insert("name".to_string(), "  JOHN DOE  ".to_string());
        row.insert("email".to_string(), "  TEST@EXAMPLE.COM  ".to_string());

        cleaner.clean_row(&mut row);

        assert_eq!(row.get("name").unwrap(), "john doe");
        assert_eq!(row.get("email").unwrap(), "test@example.com");
        assert_eq!(row.get("age").unwrap(), "0");
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        cleaner.set_default("status", "pending");

        let mut dataset = vec![
            HashMap::from([
                ("id".to_string(), "1".to_string()),
                ("name".to_string(), "  Alice  ".to_string()),
            ]),
            HashMap::from([
                ("id".to_string(), "2".to_string()),
                ("name".to_string(), "".to_string()),
            ]),
        ];

        cleaner.clean_dataset(&mut dataset);

        assert_eq!(dataset[0].get("name").unwrap(), "alice");
        assert_eq!(dataset[0].get("status").unwrap(), "pending");
        assert_eq!(dataset[1].get("name").unwrap(), "");
        assert_eq!(dataset[1].get("status").unwrap(), "pending");
    }
}