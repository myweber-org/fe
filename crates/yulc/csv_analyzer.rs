
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    file_path: String,
    delimiter: char,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = header_line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        let mut column_stats: HashMap<String, ColumnStatistics> = HashMap::new();
        let mut total_rows = 0;
        let mut malformed_rows = 0;

        for header in &headers {
            column_stats.insert(header.clone(), ColumnStatistics::new());
        }

        for line_result in lines {
            let line = line_result?;
            total_rows += 1;

            let values: Vec<&str> = line.split(self.delimiter).collect();
            
            if values.len() != headers.len() {
                malformed_rows += 1;
                continue;
            }

            for (i, value) in values.iter().enumerate() {
                let header = &headers[i];
                if let Some(stats) = column_stats.get_mut(header) {
                    stats.process_value(value);
                }
            }
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            total_rows,
            malformed_rows,
            headers: headers.clone(),
            column_stats,
        })
    }
}

pub struct AnalysisResult {
    file_path: String,
    total_rows: usize,
    malformed_rows: usize,
    headers: Vec<String>,
    column_stats: HashMap<String, ColumnStatistics>,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Report");
        println!("===================");
        println!("File: {}", self.file_path);
        println!("Total rows: {}", self.total_rows);
        println!("Malformed rows: {}", self.malformed_rows);
        println!("Headers: {}", self.headers.join(", "));
        println!("\nColumn Statistics:");
        
        for (header, stats) in &self.column_stats {
            println!("\n{}:", header);
            println!("  Non-empty values: {}", stats.non_empty_count);
            println!("  Empty values: {}", stats.empty_count);
            println!("  Numeric values: {}", stats.numeric_count);
            println!("  Unique values: {}", stats.unique_values.len());
        }
    }
}

struct ColumnStatistics {
    non_empty_count: usize,
    empty_count: usize,
    numeric_count: usize,
    unique_values: std::collections::HashSet<String>,
}

impl ColumnStatistics {
    fn new() -> Self {
        ColumnStatistics {
            non_empty_count: 0,
            empty_count: 0,
            numeric_count: 0,
            unique_values: std::collections::HashSet::new(),
        }
    }

    fn process_value(&mut self, value: &str) {
        let trimmed = value.trim();
        
        if trimmed.is_empty() {
            self.empty_count += 1;
        } else {
            self.non_empty_count += 1;
            self.unique_values.insert(trimmed.to_string());
            
            if trimmed.parse::<f64>().is_ok() {
                self.numeric_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let csv_content = "name,age,city\nJohn,25,New York\nJane,30,London\nBob,,Paris\n";
        write!(temp_file, "{}", csv_content).unwrap();
        
        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap(), ',');
        let result = analyzer.analyze().unwrap();
        
        assert_eq!(result.total_rows, 3);
        assert_eq!(result.malformed_rows, 0);
        assert_eq!(result.headers, vec!["name", "age", "city"]);
        
        let age_stats = result.column_stats.get("age").unwrap();
        assert_eq!(age_stats.numeric_count, 2);
        assert_eq!(age_stats.empty_count, 1);
    }
}