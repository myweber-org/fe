
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
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

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            self.records
                .iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect()
        })
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let sum: f64 = self.records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .sum();
        
        Some(sum)
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(f64, f64, f64)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse::<f64>().ok())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        Some((mean, variance, variance.sqrt()))
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value").unwrap();
        writeln!(file, "1,item_a,10.5").unwrap();
        writeln!(file, "2,item_b,20.3").unwrap();
        writeln!(file, "3,item_a,15.7").unwrap();
        writeln!(file, "4,item_c,5.2").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), vec!["id", "name", "value"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("name", |name| name == "item_a");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let sum = processor.aggregate_numeric_column("value").unwrap();
        assert!((sum - 51.7).abs() < 0.001);
    }

    #[test]
    fn test_stats_calculation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("value").unwrap();
        assert!((stats.0 - 12.925).abs() < 0.001);
    }
}