
use std::collections::HashMap;

pub struct DataCleaner {
    data: HashMap<String, Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: HashMap<String, Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn clean(&mut self) -> HashMap<String, Vec<String>> {
        let mut cleaned_data = HashMap::new();

        for (column, values) in &self.data {
            let cleaned_values: Vec<String> = values
                .iter()
                .filter_map(|val| {
                    val.as_ref().map(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    })
                })
                .flatten()
                .collect();

            if !cleaned_values.is_empty() {
                cleaned_data.insert(column.clone(), cleaned_values);
            }
        }

        cleaned_data
    }

    pub fn remove_columns_with_all_null(&mut self) -> Vec<String> {
        let mut removed_columns = Vec::new();
        let mut columns_to_remove = Vec::new();

        for (column, values) in &self.data {
            let all_null = values.iter().all(|val| val.is_none());
            if all_null {
                columns_to_remove.push(column.clone());
            }
        }

        for column in columns_to_remove {
            self.data.remove(&column);
            removed_columns.push(column);
        }

        removed_columns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_nulls_and_trims() {
        let mut data = HashMap::new();
        data.insert(
            "name".to_string(),
            vec![
                Some("  John  ".to_string()),
                None,
                Some("Jane".to_string()),
                Some("  ".to_string()),
            ],
        );

        let mut cleaner = DataCleaner::new(data);
        let cleaned = cleaner.clean();

        assert_eq!(cleaned.get("name").unwrap(), &vec!["John", "Jane"]);
    }

    #[test]
    fn test_remove_all_null_columns() {
        let mut data = HashMap::new();
        data.insert("valid".to_string(), vec![Some("data".to_string())]);
        data.insert("empty".to_string(), vec![None, None, None]);

        let mut cleaner = DataCleaner::new(data);
        let removed = cleaner.remove_columns_with_all_null();

        assert_eq!(removed, vec!["empty"]);
        assert_eq!(cleaner.data.len(), 1);
        assert!(cleaner.data.contains_key("valid"));
    }
}