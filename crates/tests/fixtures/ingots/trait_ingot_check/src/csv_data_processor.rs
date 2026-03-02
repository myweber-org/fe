
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,75.2,Books").unwrap();
        writeln!(temp_file, "3,ItemC,120.8,Electronics").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(file_path);
        assert!(result.is_ok());
        
        assert_eq!(processor.get_record_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg_value = processor.calculate_average_value();
        assert!((avg_value - 98.833).abs() < 0.001);
        
        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 120.8);
        
        let categories = processor.get_categories();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], "Books");
        assert_eq!(categories[1], "Electronics");
    }
}use std::error::Error;
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
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| record.get(col_index).map_or(false, |v| v == value))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let sum: f64 = self.records
                .iter()
                .filter_map(|record| record.get(col_index).and_then(|v| v.parse::<f64>().ok()))
                .sum();
            
            if !self.records.is_empty() {
                Some(sum / self.records.len() as f64)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let mut values: Vec<String> = self.records
                .iter()
                .filter_map(|record| record.get(col_index).cloned())
                .collect();
            
            values.sort();
            values.dedup();
            values
        } else {
            Vec::new()
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
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
        writeln!(file, "Alice,25,London").unwrap();
        writeln!(file, "Bob,30,Paris").unwrap();
        writeln!(file, "Charlie,25,London").unwrap();
        writeln!(file, "Diana,35,Tokyo").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.column_count(), 3);
        assert_eq!(processor.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let london_records = processor.filter_by_column("city", "London");
        assert_eq!(london_records.len(), 2);
        
        let paris_records = processor.filter_by_column("city", "Paris");
        assert_eq!(paris_records.len(), 1);
    }

    #[test]
    fn test_aggregate_numeric() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let avg_age = processor.aggregate_numeric_column("age").unwrap();
        assert!((avg_age - 28.75).abs() < 0.001);
    }

    #[test]
    fn test_unique_values() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let unique_cities = processor.get_unique_values("city");
        assert_eq!(unique_cities, vec!["London", "Paris", "Tokyo"]);
    }
}