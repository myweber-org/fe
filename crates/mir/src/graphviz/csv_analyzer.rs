use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
    pub sample_rows: Vec<Vec<String>>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => return Err(Box::new(e)),
        None => return Err("Empty file".into()),
    };

    let columns: Vec<String> = first_line.split(',').map(|s| s.trim().to_string()).collect();
    let column_count = columns.len();
    
    let mut row_count = 1;
    let mut sample_rows = Vec::with_capacity(sample_size);
    sample_rows.push(columns.clone());

    let mut has_header = true;
    for column in &columns {
        if column.parse::<f64>().is_ok() {
            has_header = false;
            break;
        }
    }

    for line in lines.take(sample_size - 1) {
        let line = line?;
        let row: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        
        if row.len() != column_count {
            return Err(format!("Row {} has {} columns, expected {}", row_count + 1, row.len(), column_count).into());
        }
        
        sample_rows.push(row);
        row_count += 1;
    }

    for line in lines {
        let _ = line?;
        row_count += 1;
    }

    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_rows,
    })
}

pub fn validate_csv_format<P: AsRef<Path>>(file_path: P) -> Result<bool, Box<dyn Error>> {
    let stats = analyze_csv(file_path, 10)?;
    
    if stats.row_count == 0 {
        return Ok(false);
    }
    
    if stats.column_count == 0 {
        return Ok(false);
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_valid_csv() {
        let csv = "name,age,city\nAlice,30,New York\nBob,25,London";
        let file = create_test_csv(csv);
        
        let stats = analyze_csv(file.path(), 5).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
        assert_eq!(stats.sample_rows.len(), 3);
    }

    #[test]
    fn test_csv_without_header() {
        let csv = "Alice,30,New York\nBob,25,London\nCharlie,35,Paris";
        let file = create_test_csv(csv);
        
        let stats = analyze_csv(file.path(), 5).unwrap();
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 3);
        assert!(!stats.has_header);
    }

    #[test]
    fn test_invalid_column_count() {
        let csv = "name,age,city\nAlice,30\nBob,25,London,extra";
        let file = create_test_csv(csv);
        
        let result = analyze_csv(file.path(), 5);
        assert!(result.is_err());
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub column_names: Vec<String>,
    pub column_types: HashMap<String, String>,
    pub numeric_columns: Vec<String>,
}

pub struct CsvAnalyzer {
    path: String,
    delimiter: char,
}

impl CsvAnalyzer {
    pub fn new(path: &str) -> Self {
        CsvAnalyzer {
            path: path.to_string(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn analyze(&self) -> Result<CsvStats, Box<dyn Error>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header_line = lines.next().ok_or("Empty file")??;
        let column_names: Vec<String> = header_line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        let mut column_types = HashMap::new();
        let mut numeric_columns = Vec::new();
        let mut row_count = 0;

        for line in lines {
            let line = line?;
            let values: Vec<&str> = line.split(self.delimiter).collect();

            if values.len() != column_names.len() {
                continue;
            }

            for (i, value) in values.iter().enumerate() {
                let col_name = &column_names[i];
                let current_type = column_types.entry(col_name.clone()).or_insert("unknown".to_string());

                if *current_type == "unknown" {
                    if value.parse::<f64>().is_ok() {
                        *current_type = "numeric".to_string();
                        numeric_columns.push(col_name.clone());
                    } else if !value.is_empty() {
                        *current_type = "text".to_string();
                    }
                } else if *current_type == "numeric" && value.parse::<f64>().is_err() && !value.is_empty() {
                    *current_type = "text".to_string();
                    numeric_columns.retain(|x| x != col_name);
                }
            }

            row_count += 1;
        }

        Ok(CsvStats {
            row_count,
            column_count: column_names.len(),
            column_names,
            column_types,
            numeric_columns,
        })
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header_line = lines.next().ok_or("Empty file")??;
        let column_names: Vec<String> = header_line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        let mut filtered_rows = Vec::new();

        for line in lines {
            let line = line?;
            let values: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if values.len() == column_names.len() && predicate(&values) {
                filtered_rows.push(values);
            }
        }

        Ok(filtered_rows)
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
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        writeln!(temp_file, "3,Charlie,35,60000").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let stats = analyzer.analyze().unwrap();

        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.column_names, vec!["id", "name", "age", "salary"]);
        assert_eq!(stats.numeric_columns.len(), 3);
    }

    #[test]
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Charlie,35").unwrap();

        let analyzer = CsvAnalyzer::new(temp_file.path().to_str().unwrap());
        let filtered = analyzer
            .filter_rows(|row| row[2].parse::<i32>().unwrap_or(0) > 30)
            .unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][1], "Charlie");
    }
}