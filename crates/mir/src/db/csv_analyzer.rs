
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

pub struct CsvAnalyzer {
    pub records: Vec<CsvRecord>,
    pub headers: Vec<String>,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line.split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let values: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            if values.len() == headers.len() {
                records.push(CsvRecord {
                    columns: headers.clone(),
                    values,
                });
            }
        }

        Ok(CsvAnalyzer { records, headers })
    }

    pub fn column_stats(&self, column_name: &str) -> Option<HashMap<String, usize>> {
        let col_index = self.headers.iter().position(|h| h == column_name)?;
        let mut stats = HashMap::new();

        for record in &self.records {
            if let Some(value) = record.values.get(col_index) {
                *stats.entry(value.clone()).or_insert(0) += 1;
            }
        }

        Some(stats)
    }

    pub fn filter_by_value(&self, column_name: &str, target_value: &str) -> Vec<&CsvRecord> {
        let col_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records.iter()
            .filter(|record| {
                record.values.get(col_index)
                    .map(|val| val == target_value)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn numeric_column_summary(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let col_index = self.headers.iter().position(|h| h == column_name)?;
        let mut numeric_values = Vec::new();

        for record in &self.records {
            if let Some(value_str) = record.values.get(col_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    numeric_values.push(value);
                }
            }
        }

        if numeric_values.is_empty() {
            return None;
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let avg = sum / count;
        let max = numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        Some((avg, min, max))
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
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.headers, vec!["name", "age", "city"]);
        assert_eq!(analyzer.records.len(), 4);
    }

    #[test]
    fn test_column_stats() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        let stats = analyzer.column_stats("city").unwrap();
        
        assert_eq!(stats.get("London"), Some(&2));
        assert_eq!(stats.get("Paris"), Some(&1));
    }

    #[test]
    fn test_filter_by_value() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        let filtered = analyzer.filter_by_value("city", "London");
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.values[2] == "London"));
    }

    #[test]
    fn test_numeric_summary() {
        let test_file = create_test_csv();
        let analyzer = CsvAnalyzer::new(test_file.path().to_str().unwrap()).unwrap();
        let summary = analyzer.numeric_column_summary("age").unwrap();
        
        assert_eq!(summary.0, 28.75); // average
        assert_eq!(summary.1, 25.0);  // min
        assert_eq!(summary.2, 35.0);  // max
    }
}