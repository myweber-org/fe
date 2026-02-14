
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvFilter {
    delimiter: char,
    has_header: bool,
}

impl CsvFilter {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvFilter {
            delimiter,
            has_header,
        }
    }

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut filtered_rows = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&columns) {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn extract_column(&self, rows: &[Vec<String>], column_index: usize) -> Vec<String> {
        rows.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_extract() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let filter = CsvFilter::new(',', true);
        let filtered = filter
            .filter_rows(temp_file.path(), |row| {
                row.get(1)
                    .and_then(|age_str| age_str.parse::<u32>().ok())
                    .map_or(false, |age| age >= 30)
            })
            .unwrap();

        assert_eq!(filtered.len(), 2);
        
        let names = filter.extract_column(&filtered, 0);
        assert!(names.contains(&"Alice".to_string()));
        assert!(names.contains(&"Charlie".to_string()));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn filter_csv_by_column(
    file_path: &str,
    column_index: usize,
    filter_value: &str,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_rows = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let columns: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
        
        if columns.get(column_index).map(|s| s.as_str()) == Some(filter_value) {
            filtered_rows.push(columns);
        }
    }

    Ok(filtered_rows)
}

pub fn calculate_column_average(
    file_path: &str,
    column_index: usize,
) -> Result<f64, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut sum = 0.0;
    let mut count = 0;

    for line in reader.lines().skip(1) {
        let line = line?;
        let columns: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
        
        if let Some(value_str) = columns.get(column_index) {
            if let Ok(value) = value_str.parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        Ok(sum / count as f64)
    } else {
        Ok(0.0)
    }
}