
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut records = Vec::new();
        for line in lines {
            let line = line?;
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
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        self.records.iter()
            .filter(|record| {
                record.get(column_index)
                    .map(|value| predicate(value))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn group_by_column(&self, column_name: &str) -> HashMap<String, Vec<Vec<String>>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };
        
        let mut groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        
        for record in &self.records {
            if let Some(key) = record.get(column_index) {
                groups.entry(key.clone())
                    .or_insert_with(Vec::new)
                    .push(record.clone());
            }
        }
        
        groups
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() > 30);
        assert_eq!(filtered.len(), 1);
        
        let avg_salary = processor.aggregate_numeric_column("salary", "avg");
        assert!(avg_salary.is_some());
        assert!((avg_salary.unwrap() - 51666.666).abs() < 0.1);
        
        let groups = processor.group_by_column("name");
        assert_eq!(groups.len(), 3);
    }
}