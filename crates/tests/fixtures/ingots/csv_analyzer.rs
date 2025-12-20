use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub numeric_columns: HashMap<String, Vec<f64>>,
    pub text_columns: HashMap<String, Vec<String>>,
}

impl CsvStats {
    pub fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_names: Vec::new(),
            numeric_columns: HashMap::new(),
            text_columns: HashMap::new(),
        }
    }

    pub fn analyze_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CsvStats::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                stats.column_names = line.split(',').map(|s| s.trim().to_string()).collect();
                stats.column_count = stats.column_names.len();
                continue;
            }

            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != stats.column_count {
                return Err(format!("Row {} has {} columns, expected {}", 
                    index + 1, values.len(), stats.column_count).into());
            }

            for (col_index, value) in values.iter().enumerate() {
                let column_name = &stats.column_names[col_index];
                
                if let Ok(num) = value.parse::<f64>() {
                    stats.numeric_columns
                        .entry(column_name.clone())
                        .or_insert_with(Vec::new)
                        .push(num);
                } else {
                    stats.text_columns
                        .entry(column_name.clone())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
            }
            
            stats.row_count += 1;
        }

        Ok(stats)
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<ColumnStats> {
        if let Some(numbers) = self.numeric_columns.get(column_name) {
            if numbers.is_empty() {
                return None;
            }

            let sum: f64 = numbers.iter().sum();
            let count = numbers.len();
            let mean = sum / count as f64;
            
            let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let variance: f64 = numbers.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            Some(ColumnStats::Numeric {
                count,
                mean,
                min,
                max,
                variance,
                std_dev: variance.sqrt(),
            })
        } else if let Some(texts) = self.text_columns.get(column_name) {
            let unique_count = texts.iter().collect::<std::collections::HashSet<_>>().len();
            let max_length = texts.iter().map(|s| s.len()).max().unwrap_or(0);
            let min_length = texts.iter().map(|s| s.len()).min().unwrap_or(0);
            
            Some(ColumnStats::Text {
                count: texts.len(),
                unique_count,
                max_length,
                min_length,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum ColumnStats {
    Numeric {
        count: usize,
        mean: f64,
        min: f64,
        max: f64,
        variance: f64,
        std_dev: f64,
    },
    Text {
        count: usize,
        unique_count: usize,
        max_length: usize,
        min_length: usize,
    },
}

pub fn validate_csv_format(path: &str) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        Some(Err(e)) => return Err(e.into()),
        None => return Err("Empty file".into()),
    };
    
    let column_count = header.split(',').count();
    let mut line_number = 1;
    
    for line in lines {
        let line = line?;
        line_number += 1;
        
        let current_count = line.split(',').count();
        if current_count != column_count {
            return Err(format!("Line {}: expected {} columns, found {}", 
                line_number, column_count, current_count).into());
        }
    }
    
    Ok(true)
}