use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvParser {
    delimiter: char,
    has_headers: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvParser {
            delimiter,
            has_headers,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _headers = lines.next().transpose()?;
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn get_column(&self, data: &[Vec<String>], column_index: usize) -> Option<Vec<String>> {
        let mut column_data = Vec::new();
        
        for row in data {
            if let Some(value) = row.get(column_index) {
                column_data.push(value.clone());
            } else {
                return None;
            }
        }
        
        Some(column_data)
    }
}

pub fn calculate_average(numbers: &[String]) -> Result<f64, Box<dyn Error>> {
    let mut sum = 0.0;
    let mut count = 0;
    
    for num_str in numbers {
        let num: f64 = num_str.parse()?;
        sum += num;
        count += 1;
    }
    
    if count == 0 {
        return Err("No numbers to calculate average".into());
    }
    
    Ok(sum / count as f64)
}