use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Result<f64, Box<dyn Error>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Err("Column not found".into()),
        };

        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No numeric values found".into())
        }
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(index) => index,
            None => return Vec::new(),
        };

        let mut unique_values = std::collections::HashSet::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                unique_values.insert(value.clone());
            }
        }

        unique_values.into_iter().collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn header_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,New York").unwrap();
        writeln!(file, "Diana,28,Paris").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.header_count(), 3);
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("city", "New York");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }

    #[test]
    fn test_aggregate_numeric() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let average_age = processor.aggregate_numeric_column("age").unwrap();
        assert!((average_age - 29.5).abs() < 0.001);
    }

    #[test]
    fn test_unique_values() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let unique_cities = processor.get_unique_values("city");
        assert_eq!(unique_cities.len(), 3);
        assert!(unique_cities.contains(&"New York".to_string()));
        assert!(unique_cities.contains(&"London".to_string()));
        assert!(unique_cities.contains(&"Paris".to_string()));
    }
}