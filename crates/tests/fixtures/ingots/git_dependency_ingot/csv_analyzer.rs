use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
    row_count: usize,
    column_count: usize,
}

impl CsvAnalyzer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
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
        
        let mut data = Vec::new();
        let mut row_count = 0;
        
        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            if row.len() != headers.len() {
                return Err(format!(
                    "Row {} has {} columns, expected {}", 
                    row_count + 1, 
                    row.len(), 
                    headers.len()
                ).into());
            }
            
            data.push(row);
            row_count += 1;
        }
        
        Ok(CsvAnalyzer {
            headers,
            data,
            row_count,
            column_count: headers.len(),
        })
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn get_row_count(&self) -> usize {
        self.row_count
    }
    
    pub fn get_column_count(&self) -> usize {
        self.column_count
    }
    
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        if row < self.row_count && col < self.column_count {
            Some(&self.data[row][col])
        } else {
            None
        }
    }
    
    pub fn get_column_data(&self, column_name: &str) -> Option<Vec<&str>> {
        let col_index = self.headers.iter().position(|h| h == column_name)?;
        
        let column_data: Vec<&str> = self.data
            .iter()
            .map(|row| row[col_index].as_str())
            .collect();
        
        Some(column_data)
    }
    
    pub fn find_rows_by_value(&self, column_name: &str, value: &str) -> Vec<Vec<&str>> {
        let mut result = Vec::new();
        
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            for row in &self.data {
                if row[col_index] == value {
                    let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
                    result.push(row_refs);
                }
            }
        }
        
        result
    }
    
    pub fn calculate_column_stats(&self, column_name: &str) -> Option<ColumnStats> {
        let column_data = self.get_column_data(column_name)?;
        
        let numeric_values: Vec<f64> = column_data
            .iter()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        
        if numeric_values.is_empty() {
            return None;
        }
        
        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len();
        let mean = sum / count as f64;
        
        let variance: f64 = numeric_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some(ColumnStats {
            column_name: column_name.to_string(),
            count,
            numeric_count: numeric_values.len(),
            mean,
            std_dev,
            min,
            max,
            sum,
        })
    }
}

pub struct ColumnStats {
    pub column_name: String,
    pub count: usize,
    pub numeric_count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Statistics for column: {}", self.column_name)?;
        writeln!(f, "  Total rows: {}", self.count)?;
        writeln!(f, "  Numeric values: {}", self.numeric_count)?;
        writeln!(f, "  Mean: {:.4}", self.mean)?;
        writeln!(f, "  Std Dev: {:.4}", self.std_dev)?;
        writeln!(f, "  Min: {:.4}", self.min)?;
        writeln!(f, "  Max: {:.4}", self.max)?;
        writeln!(f, "  Sum: {:.4}", self.sum)
    }
}