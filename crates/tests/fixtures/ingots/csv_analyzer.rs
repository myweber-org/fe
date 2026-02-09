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
}use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_types: HashMap<String, String>,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty CSV file".into()),
    };
    
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let mut column_samples: HashMap<String, Vec<String>> = HashMap::new();
    let mut row_count = 0;
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if values.len() != headers.len() {
            continue;
        }
        
        for (i, value) in values.iter().enumerate() {
            column_samples
                .entry(headers[i].clone())
                .or_insert_with(Vec::new)
                .push(value.to_string());
        }
        
        row_count += 1;
    }
    
    let mut column_types = HashMap::new();
    let mut numeric_columns = Vec::new();
    let mut text_columns = Vec::new();
    
    for (header, samples) in column_samples {
        let is_numeric = samples.iter().all(|s| s.parse::<f64>().is_ok());
        let col_type = if is_numeric { "numeric" } else { "text" };
        
        column_types.insert(header.clone(), col_type.to_string());
        
        if is_numeric {
            numeric_columns.push(header);
        } else {
            text_columns.push(header);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count: headers.len(),
        column_types,
        numeric_columns,
        text_columns,
    })
}

pub fn filter_csv_rows<F>(
    file_path: &str,
    predicate: F,
) -> Result<Vec<Vec<String>>, Box<dyn Error>>
where
    F: Fn(&[String]) -> bool,
{
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Ok(Vec::new()),
    };
    
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let mut filtered_rows = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        let values: Vec<String> = line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if values.len() == headers.len() && predicate(&values) {
            filtered_rows.push(values);
        }
    }
    
    Ok(filtered_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,60000").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns.len(), 3);
        assert_eq!(stats.text_columns.len(), 1);
    }
    
    #[test]
    fn test_filter_csv_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Charlie,35").unwrap();
        
        let filtered = filter_csv_rows(
            temp_file.path().to_str().unwrap(),
            |row| row[2].parse::<i32>().unwrap_or(0) > 30
        ).unwrap();
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][1], "Charlie");
    }
}