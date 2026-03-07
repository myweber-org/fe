
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
    column_types: HashMap<String, DataType>,
}

#[derive(Debug, Clone)]
enum DataType {
    Integer,
    Float,
    String,
    Boolean,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut data = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        let mut analyzer = CsvAnalyzer {
            headers,
            data,
            column_types: HashMap::new(),
        };
        
        analyzer.infer_column_types();
        Ok(analyzer)
    }
    
    fn infer_column_types(&mut self) {
        if self.data.is_empty() {
            return;
        }
        
        for (col_idx, header) in self.headers.iter().enumerate() {
            let mut samples = Vec::new();
            for row in &self.data {
                if col_idx < row.len() {
                    samples.push(&row[col_idx]);
                }
            }
            
            let data_type = Self::detect_type_from_samples(&samples);
            self.column_types.insert(header.clone(), data_type);
        }
    }
    
    fn detect_type_from_samples(samples: &[&String]) -> DataType {
        let mut int_count = 0;
        let mut float_count = 0;
        let mut bool_count = 0;
        
        for sample in samples {
            if sample.parse::<i64>().is_ok() {
                int_count += 1;
            } else if sample.parse::<f64>().is_ok() {
                float_count += 1;
            } else if sample.eq_ignore_ascii_case("true") || 
                      sample.eq_ignore_ascii_case("false") ||
                      sample.eq_ignore_ascii_case("yes") || 
                      sample.eq_ignore_ascii_case("no") {
                bool_count += 1;
            }
        }
        
        let total = samples.len();
        if total > 0 {
            let int_ratio = int_count as f32 / total as f32;
            let float_ratio = float_count as f32 / total as f32;
            let bool_ratio = bool_count as f32 / total as f32;
            
            if bool_ratio > 0.8 {
                DataType::Boolean
            } else if int_ratio > 0.9 {
                DataType::Integer
            } else if float_ratio > 0.9 {
                DataType::Float
            } else {
                DataType::String
            }
        } else {
            DataType::String
        }
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
    
    pub fn get_column_summary(&self, column_name: &str) -> Option<ColumnSummary> {
        let col_idx = self.headers.iter().position(|h| h == column_name)?;
        let data_type = self.column_types.get(column_name)?;
        
        let mut numeric_values = Vec::new();
        let mut string_values = Vec::new();
        
        for row in &self.data {
            if col_idx < row.len() {
                let value = &row[col_idx];
                match data_type {
                    DataType::Integer => {
                        if let Ok(num) = value.parse::<i64>() {
                            numeric_values.push(num as f64);
                        }
                    }
                    DataType::Float => {
                        if let Ok(num) = value.parse::<f64>() {
                            numeric_values.push(num);
                        }
                    }
                    DataType::Boolean => {
                        let bool_val = value.eq_ignore_ascii_case("true") || 
                                      value.eq_ignore_ascii_case("yes");
                        numeric_values.push(if bool_val { 1.0 } else { 0.0 });
                    }
                    DataType::String => {
                        string_values.push(value.clone());
                    }
                }
            }
        }
        
        Some(ColumnSummary::new(
            column_name,
            data_type.clone(),
            numeric_values,
            string_values,
        ))
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn get_unique_values(&self, column_name: &str) -> Option<Vec<String>> {
        let col_idx = self.headers.iter().position(|h| h == column_name)?;
        let mut unique_values = std::collections::HashSet::new();
        
        for row in &self.data {
            if col_idx < row.len() {
                unique_values.insert(row[col_idx].clone());
            }
        }
        
        let mut result: Vec<String> = unique_values.into_iter().collect();
        result.sort();
        Some(result)
    }
}

#[derive(Debug)]
pub struct ColumnSummary {
    name: String,
    data_type: DataType,
    count: usize,
    unique_count: usize,
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    median: Option<f64>,
    mode: Option<String>,
}

impl ColumnSummary {
    fn new(
        name: &str,
        data_type: DataType,
        numeric_values: Vec<f64>,
        string_values: Vec<String>,
    ) -> Self {
        let count = numeric_values.len().max(string_values.len());
        let unique_count = if !string_values.is_empty() {
            let unique_strings: std::collections::HashSet<_> = string_values.iter().collect();
            unique_strings.len()
        } else {
            let unique_nums: std::collections::HashSet<_> = numeric_values.iter().collect();
            unique_nums.len()
        };
        
        let (min, max, mean, median) = if !numeric_values.is_empty() {
            let mut sorted = numeric_values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let min = sorted.first().copied();
            let max = sorted.last().copied();
            let mean = if !sorted.is_empty() {
                Some(sorted.iter().sum::<f64>() / sorted.len() as f64)
            } else {
                None
            };
            let median = if !sorted.is_empty() {
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    Some((sorted[mid - 1] + sorted[mid]) / 2.0)
                } else {
                    sorted.get(mid).copied()
                }
            } else {
                None
            };
            
            (min, max, mean, median)
        } else {
            (None, None, None, None)
        };
        
        let mode = if !string_values.is_empty() {
            let mut frequency = HashMap::new();
            for value in &string_values {
                *frequency.entry(value.clone()).or_insert(0) += 1;
            }
            frequency.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(value, _)| value)
        } else {
            None
        };
        
        ColumnSummary {
            name: name.to_string(),
            data_type,
            count,
            unique_count,
            min,
            max,
            mean,
            median,
            mode,
        }
    }
    
    pub fn print_summary(&self) {
        println!("Column: {}", self.name);
        println!("  Data Type: {:?}", self.data_type);
        println!("  Total Values: {}", self.count);
        println!("  Unique Values: {}", self.unique_count);
        
        if let Some(min) = self.min {
            println!("  Minimum: {:.2}", min);
        }
        if let Some(max) = self.max {
            println!("  Maximum: {:.2}", max);
        }
        if let Some(mean) = self.mean {
            println!("  Mean: {:.2}", mean);
        }
        if let Some(median) = self.median {
            println!("  Median: {:.2}", median);
        }
        if let Some(mode) = &self.mode {
            println!("  Mode: {}", mode);
        }
    }
}