
use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRow {
    pub id: u32,
    pub values: Vec<Option<f64>>,
}

pub struct DataCleaner;

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner
    }

    pub fn remove_null_rows(&self, data: &mut Vec<DataRow>) -> Vec<DataRow> {
        data.into_iter()
            .filter(|row| row.values.iter().all(|val| val.is_some()))
            .cloned()
            .collect()
    }

    pub fn deduplicate_by_id(&self, data: &[DataRow]) -> Vec<DataRow> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for row in data {
            if seen.insert(row.id) {
                result.push(row.clone());
            }
        }

        result
    }

    pub fn clean_dataset(&self, data: &[DataRow]) -> Result<Vec<DataRow>, Box<dyn Error>> {
        let mut cleaned: Vec<DataRow> = self.remove_null_rows(&mut data.to_vec());
        cleaned = self.deduplicate_by_id(&cleaned);

        if cleaned.is_empty() {
            return Err("No valid data remaining after cleaning".into());
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let cleaner = DataCleaner::new();
        let mut data = vec![
            DataRow {
                id: 1,
                values: vec![Some(1.0), Some(2.0)],
            },
            DataRow {
                id: 2,
                values: vec![Some(3.0), None],
            },
        ];

        let result = cleaner.remove_null_rows(&mut data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn test_deduplicate_by_id() {
        let cleaner = DataCleaner::new();
        let data = vec![
            DataRow {
                id: 1,
                values: vec![Some(1.0)],
            },
            DataRow {
                id: 1,
                values: vec![Some(2.0)],
            },
            DataRow {
                id: 2,
                values: vec![Some(3.0)],
            },
        ];

        let result = cleaner.deduplicate_by_id(&data);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|r| r.id == 1));
        assert!(result.iter().any(|r| r.id == 2));
    }
}