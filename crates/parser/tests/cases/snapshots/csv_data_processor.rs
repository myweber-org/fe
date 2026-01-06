use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
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
        
        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.data
                .iter()
                .filter(|row| row.get(col_index).map_or(false, |v| v == value))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let sum: f64 = self.data
                .iter()
                .filter_map(|row| row.get(col_index).and_then(|v| v.parse::<f64>().ok()))
                .sum();
            
            if !self.data.is_empty() {
                Some(sum / self.data.len() as f64)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
    
    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let mut values: Vec<String> = self.data
                .iter()
                .filter_map(|row| row.get(col_index).cloned())
                .collect();
            
            values.sort();
            values.dedup();
            values
        } else {
            Vec::new()
        }
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}