
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
            let line = line?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(column_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| record.get(column_index) == Some(&value.to_string()))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        if let Some(column_index) = self.headers.iter().position(|h| h == column_name) {
            let sum: f64 = self
                .records
                .iter()
                .filter_map(|record| record.get(column_index)?.parse::<f64>().ok())
                .sum();

            if self.records.is_empty() {
                None
            } else {
                Some(sum)
            }
        } else {
            None
        }
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        if let Some(column_index) = self.headers.iter().position(|h| h == column_name) {
            let mut values: Vec<String> = self
                .records
                .iter()
                .filter_map(|record| record.get(column_index).cloned())
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "id,name,age,score\n1,Alice,25,95.5\n2,Bob,30,87.0\n3,Charlie,25,92.0\n4,Alice,28,88.5"
        )
        .unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.column_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let results = processor.filter_by_column("name", "Alice");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let total_score = processor.aggregate_numeric_column("score");
        assert_eq!(total_score, Some(363.0));
    }

    #[test]
    fn test_get_unique_values() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages, vec!["25", "28", "30"]);
    }
}