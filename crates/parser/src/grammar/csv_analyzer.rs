
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

        let mut column_stats: HashMap<String, ColumnStats> = HashMap::new();
        let mut total_rows = 0;
        let mut empty_cells = 0;

        for header in &headers {
            column_stats.insert(header.clone(), ColumnStats::new());
        }

        for line_result in lines {
            let line = line_result?;
            total_rows += 1;

            let values: Vec<&str> = line.split(self.delimiter).collect();

            for (i, value) in values.iter().enumerate() {
                if i >= headers.len() {
                    break;
                }

                let header = &headers[i];
                if let Some(stats) = column_stats.get_mut(header) {
                    let trimmed = value.trim();
                    
                    if trimmed.is_empty() {
                        empty_cells += 1;
                        stats.empty_count += 1;
                    } else {
                        stats.non_empty_count += 1;
                        
                        if let Ok(num) = trimmed.parse::<f64>() {
                            stats.numeric_count += 1;
                            stats.sum += num;
                            stats.min = stats.min.min(num);
                            stats.max = stats.max.max(num);
                        } else {
                            stats.string_count += 1;
                            let length = trimmed.len();
                            stats.avg_string_length = (stats.avg_string_length * (stats.string_count - 1) as f64 + length as f64) / stats.string_count as f64;
                        }
                    }
                }
            }
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            headers,
            total_rows,
            empty_cells,
            column_stats,
        })
    }
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub file_path: String,
    pub headers: Vec<String>,
    pub total_rows: usize,
    pub empty_cells: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

#[derive(Debug, Default)]
pub struct ColumnStats {
    pub empty_count: usize,
    pub non_empty_count: usize,
    pub numeric_count: usize,
    pub string_count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg_string_length: f64,
}

impl ColumnStats {
    fn new() -> Self {
        ColumnStats {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            ..Default::default()
        }
    }

    pub fn avg(&self) -> Option<f64> {
        if self.numeric_count > 0 {
            Some(self.sum / self.numeric_count as f64)
        } else {
            None
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
        writeln!(temp_file, "Name,Age,City").unwrap();
        writeln!(temp_file, "Alice,25,New York").unwrap();
        writeln!(temp_file, "Bob,30,London").unwrap();
        writeln!(temp_file, "Charlie,,Paris").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap(), ',');
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.headers, vec!["Name", "Age", "City"]);
        assert_eq!(result.total_rows, 3);
        
        let age_stats = result.column_stats.get("Age").unwrap();
        assert_eq!(age_stats.numeric_count, 2);
        assert_eq!(age_stats.empty_count, 1);
        assert_eq!(age_stats.avg(), Some(27.5));
    }
}