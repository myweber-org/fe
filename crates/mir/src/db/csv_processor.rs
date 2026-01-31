use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
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
            if line.trim().is_empty() {
                continue;
            }
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
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, Vec<String>)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let mut unique_values = Vec::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                if !unique_values.contains(value) {
                    unique_values.push(value.clone());
                }
            }
        }
        
        Some((unique_values.len(), unique_values))
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
        writeln!(file, "id,name,department").unwrap();
        writeln!(file, "1,Alice,Engineering").unwrap();
        writeln!(file, "2,Bob,Marketing").unwrap();
        writeln!(file, "3,Charlie,Engineering").unwrap();
        writeln!(file, "4,Diana,Marketing").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        assert_eq!(processor.header_count(), 3);
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.headers, vec!["id", "name", "department"]);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let marketing_records = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_records.len(), 2);
    }

    #[test]
    fn test_column_summary() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        let summary = processor.get_column_summary("department").unwrap();
        assert_eq!(summary.0, 2);
        assert!(summary.1.contains(&"Engineering".to_string()));
        assert!(summary.1.contains(&"Marketing".to_string()));
    }
}