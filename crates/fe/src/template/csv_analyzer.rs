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

        let header_line = match lines.next() {
            Some(Ok(line)) => line,
            _ => return Err("Empty CSV file".into()),
        };

        let headers: Vec<String> = header_line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        let mut column_stats: HashMap<String, ColumnStats> = HashMap::new();
        let mut row_count = 0;
        let mut empty_cells = 0;
        let mut malformed_rows = 0;

        for line_result in lines {
            let line = line_result?;
            row_count += 1;

            let values: Vec<&str> = line.split(self.delimiter).collect();

            if values.len() != headers.len() {
                malformed_rows += 1;
                continue;
            }

            for (i, value) in values.iter().enumerate() {
                let header = &headers[i];
                let trimmed_value = value.trim();

                if trimmed_value.is_empty() {
                    empty_cells += 1;
                }

                let stats = column_stats.entry(header.clone()).or_insert_with(ColumnStats::new);
                stats.process_value(trimmed_value);
            }
        }

        Ok(AnalysisResult {
            row_count,
            column_count: headers.len(),
            empty_cells,
            malformed_rows,
            column_stats,
        })
    }
}

pub struct AnalysisResult {
    pub row_count: usize,
    pub column_count: usize,
    pub empty_cells: usize,
    pub malformed_rows: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary:");
        println!("  Total rows: {}", self.row_count);
        println!("  Total columns: {}", self.column_count);
        println!("  Empty cells: {}", self.empty_cells);
        println!("  Malformed rows: {}", self.malformed_rows);
        println!("\nColumn Statistics:");

        for (header, stats) in &self.column_stats {
            println!("  {}:", header);
            println!("    Unique values: {}", stats.unique_count);
            println!("    Numeric values: {}", stats.numeric_count);
            println!("    Text values: {}", stats.text_count);
            if stats.numeric_count > 0 {
                println!("    Numeric range: {} - {}", stats.min_value, stats.max_value);
            }
        }
    }
}

#[derive(Clone)]
pub struct ColumnStats {
    pub unique_count: usize,
    pub numeric_count: usize,
    pub text_count: usize,
    pub min_value: f64,
    pub max_value: f64,
    unique_values: std::collections::HashSet<String>,
}

impl ColumnStats {
    fn new() -> Self {
        ColumnStats {
            unique_count: 0,
            numeric_count: 0,
            text_count: 0,
            min_value: f64::MAX,
            max_value: f64::MIN,
            unique_values: std::collections::HashSet::new(),
        }
    }

    fn process_value(&mut self, value: &str) {
        if self.unique_values.insert(value.to_string()) {
            self.unique_count += 1;
        }

        if let Ok(num) = value.parse::<f64>() {
            self.numeric_count += 1;
            self.min_value = self.min_value.min(num);
            self.max_value = self.max_value.max(num);
        } else if !value.is_empty() {
            self.text_count += 1;
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,,Paris").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap(), ',');
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.row_count, 3);
        assert_eq!(result.column_count, 3);
        assert_eq!(result.empty_cells, 1);
        assert_eq!(result.malformed_rows, 0);

        let age_stats = result.column_stats.get("age").unwrap();
        assert_eq!(age_stats.numeric_count, 2);
        assert_eq!(age_stats.min_value, 25.0);
        assert_eq!(age_stats.max_value, 30.0);
    }
}