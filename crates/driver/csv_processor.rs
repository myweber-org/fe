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

        let headers = match lines.next() {
            Some(header_line) => header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
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
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, String, String)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<&String> = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .collect();

        if values.is_empty() {
            return None;
        }

        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let sample_values = values.iter().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        
        Some((unique_count, sample_values, values.len().to_string()))
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
        writeln!(file, "id,name,status").unwrap();
        writeln!(file, "1,Alice,active").unwrap();
        writeln!(file, "2,Bob,inactive").unwrap();
        writeln!(file, "3,Charlie,active").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.header_count(), 3);
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.headers, vec!["id", "name", "status"]);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let active_records = processor.filter_by_column("status", "active");
        assert_eq!(active_records.len(), 2);
        
        let inactive_records = processor.filter_by_column("status", "inactive");
        assert_eq!(inactive_records.len(), 1);
    }

    #[test]
    fn test_column_summary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let summary = processor.get_column_summary("status").unwrap();
        assert_eq!(summary.0, 2); // unique count
        assert!(summary.1.contains("active"));
        assert_eq!(summary.2, "3"); // total count
    }
}