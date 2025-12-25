use csv::Reader;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value.is_finite() && !record.name.is_empty())
            .collect()
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let valid_count = self.validate_records().len();
        let avg_value = self.calculate_average().unwrap_or(0.0);
        let categories: std::collections::HashSet<_> = 
            self.records.iter().map(|r| r.category.clone()).collect();

        Statistics {
            total_records: count,
            valid_records: valid_count,
            average_value: avg_value,
            unique_categories: categories.len(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub average_value: f64,
    pub unique_categories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let temp_file = NamedTempFile::new().unwrap();
        let test_data = "id,name,value,category\n1,Test1,10.5,A\n2,Test2,20.0,B\n";
        std::fs::write(temp_file.path(), test_data).unwrap();

        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.records.len(), 2);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.25).abs() < 0.001);
    }
}