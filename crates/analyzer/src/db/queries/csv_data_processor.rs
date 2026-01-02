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
                .filter(|record| {
                    record.get(idx)
                        .map(|value| predicate(value))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        })
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(usize, usize, f64)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let mut count = 0;
        let mut numeric_sum = 0.0;
        let mut has_numeric = false;

        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                count += 1;
                if let Ok(num) = value.parse::<f64>() {
                    numeric_sum += num;
                    has_numeric = true;
                }
            }
        }

        let avg = if has_numeric && count > 0 {
            numeric_sum / count as f64
        } else {
            0.0
        };

        Some((count, self.records.len(), avg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000.0,Engineering").unwrap();
        writeln!(file, "Bob,25,45000.0,Marketing").unwrap();
        writeln!(file, "Charlie,35,60000.0,Engineering").unwrap();
        writeln!(file, "Diana,28,52000.0,Sales").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers, vec!["name", "age", "salary", "department"]);
        assert_eq!(processor.records.len(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", |dept| dept == "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let high_salary = processor.filter_by_column("salary", |salary| {
            salary.parse::<f64>().unwrap_or(0.0) > 50000.0
        });
        assert_eq!(high_salary.len(), 2);
    }

    #[test]
    fn test_aggregate_functions() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", "sum");
        assert_eq!(total_salary, Some(207000.0));
        
        let avg_age = processor.aggregate_numeric_column("age", "avg");
        assert_eq!(avg_age, Some(29.5));
        
        let min_salary = processor.aggregate_numeric_column("salary", "min");
        assert_eq!(min_salary, Some(45000.0));
    }

    #[test]
    fn test_column_stats() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let stats = processor.get_column_stats("salary").unwrap();
        assert_eq!(stats.0, 4); // count
        assert_eq!(stats.1, 4); // total records
        assert_eq!(stats.2, 51750.0); // average
    }
}