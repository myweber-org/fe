use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct CsvStats {
    row_count: usize,
    column_count: usize,
    column_names: Vec<String>,
    column_types: HashMap<String, String>,
    numeric_columns: Vec<String>,
}

impl CsvStats {
    fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_names: Vec::new(),
            column_types: HashMap::new(),
            numeric_columns: Vec::new(),
        }
    }

    fn analyze_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header) = lines.next() {
            let header_line = header?;
            self.column_names = header_line.split(',').map(|s| s.trim().to_string()).collect();
            self.column_count = self.column_names.len();
            
            for col in &self.column_names {
                self.column_types.insert(col.clone(), "unknown".to_string());
            }
        }

        let mut sample_data: Vec<Vec<String>> = Vec::new();
        for line in lines.take(10) {
            let line_content = line?;
            let values: Vec<String> = line_content.split(',').map(|s| s.trim().to_string()).collect();
            sample_data.push(values);
            self.row_count += 1;
        }

        self.detect_column_types(&sample_data);
        self.identify_numeric_columns();
        
        Ok(())
    }

    fn detect_column_types(&mut self, sample_data: &[Vec<String>]) {
        if sample_data.is_empty() {
            return;
        }

        for (col_idx, col_name) in self.column_names.iter().enumerate() {
            let mut is_numeric = true;
            let mut is_integer = true;
            
            for row in sample_data {
                if col_idx < row.len() {
                    let value = &row[col_idx];
                    
                    if value.parse::<f64>().is_err() {
                        is_numeric = false;
                        break;
                    }
                    
                    if value.parse::<i64>().is_err() {
                        is_integer = false;
                    }
                }
            }
            
            let col_type = if is_numeric {
                if is_integer { "integer" } else { "float" }
            } else {
                "string"
            };
            
            self.column_types.insert(col_name.clone(), col_type.to_string());
        }
    }

    fn identify_numeric_columns(&mut self) {
        for (col_name, col_type) in &self.column_types {
            if col_type == "integer" || col_type == "float" {
                self.numeric_columns.push(col_name.clone());
            }
        }
    }

    fn print_summary(&self) {
        println!("CSV Analysis Summary:");
        println!("=====================");
        println!("Rows analyzed: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("\nColumn Names:");
        for (idx, name) in self.column_names.iter().enumerate() {
            println!("  {}. {}", idx + 1, name);
        }
        println!("\nColumn Types:");
        for name in &self.column_names {
            if let Some(col_type) = self.column_types.get(name) {
                println!("  {}: {}", name, col_type);
            }
        }
        println!("\nNumeric Columns: {}", self.numeric_columns.join(", "));
    }
}

fn filter_csv_rows(file_path: &str, column_name: &str, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let mut result = Vec::new();
    
    if let Some(header) = lines.next() {
        let header_line = header?;
        let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
        
        let col_index = headers.iter().position(|h| h == column_name);
        
        if let Some(idx) = col_index {
            result.push(headers.clone());
            
            for line in lines {
                let line_content = line?;
                let values: Vec<String> = line_content.split(',').map(|s| s.trim().to_string()).collect();
                
                if idx < values.len() && values[idx] == filter_value {
                    result.push(values);
                }
            }
        } else {
            return Err(format!("Column '{}' not found", column_name).into());
        }
    }
    
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    
    let mut analyzer = CsvStats::new();
    analyzer.analyze_file(file_path)?;
    analyzer.print_summary();
    
    println!("\nFiltering rows where 'status' equals 'active':");
    let filtered = filter_csv_rows(file_path, "status", "active")?;
    
    for row in filtered.iter().take(5) {
        println!("{:?}", row);
    }
    
    if filtered.len() > 5 {
        println!("... and {} more rows", filtered.len() - 5);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let csv_content = "id,name,age,salary\n1,Alice,30,50000.5\n2,Bob,25,45000.0\n3,Charlie,35,55000.75\n";
        write!(temp_file, "{}", csv_content).unwrap();
        
        let mut analyzer = CsvStats::new();
        let result = analyzer.analyze_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(analyzer.row_count, 3);
        assert_eq!(analyzer.column_count, 4);
        assert_eq!(analyzer.column_names, vec!["id", "name", "age", "salary"]);
        assert_eq!(analyzer.numeric_columns.len(), 3);
    }

    #[test]
    fn test_filter_function() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let csv_content = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Charlie,active\n";
        write!(temp_file, "{}", csv_content).unwrap();
        
        let result = filter_csv_rows(temp_file.path().to_str().unwrap(), "status", "active");
        
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[1][2], "active");
        assert_eq!(filtered[2][2], "active");
    }
}