use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
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

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Result<f64, String> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        match operation {
            "sum" => Ok(numeric_values.iter().sum()),
            "avg" => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "min" => Ok(numeric_values
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b))),
            "max" => Ok(numeric_values
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            _ => Err(format!("Unsupported operation: {}", operation)),
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Result<(usize, f64, f64, f64), String> {
        let column_index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        let count = numeric_values.len();
        let sum: f64 = numeric_values.iter().sum();
        let avg = sum / count as f64;
        let min = numeric_values
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = numeric_values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok((count, sum, avg, min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,60000").unwrap();
        writeln!(temp_file, "4,David,40,55000").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers, vec!["id", "name", "age", "salary"]);
        assert_eq!(processor.records.len(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() > 30);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let sum = processor.aggregate_numeric_column("salary", "sum").unwrap();
        assert_eq!(sum, 210000.0);
        
        let avg = processor.aggregate_numeric_column("salary", "avg").unwrap();
        assert_eq!(avg, 52500.0);
    }

    #[test]
    fn test_column_stats() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("age").unwrap();
        assert_eq!(stats.0, 4); // count
        assert_eq!(stats.1, 130.0); // sum
        assert_eq!(stats.2, 32.5); // avg
        assert_eq!(stats.3, 25.0); // min
        assert_eq!(stats.4, 40.0); // max
    }
}