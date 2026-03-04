use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, header_line)) = lines.next() {
            let headers = header_line?;
            println!("Headers: {}", headers);
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields: Vec<String> = line
            .split(config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.iter().any(|f| f.is_empty()) {
            return Err(format!("Empty field detected at line {}", line_num + 1).into());
        }

        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found in CSV file".into());
    }

    Ok(records)
}

pub fn validate_record_lengths(records: &[Vec<String>]) -> Result<usize, Box<dyn Error>> {
    if records.is_empty() {
        return Err("Empty records provided".into());
    }

    let expected_len = records[0].len();
    for (idx, record) in records.iter().enumerate() {
        if record.len() != expected_len {
            return Err(format!(
                "Record {} has {} fields, expected {}",
                idx + 1,
                record.len(),
                expected_len
            )
            .into());
        }
    }

    Ok(expected_len)
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn get_column_summary(&self, column_name: &str) -> Option<(usize, Vec<String>)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let mut unique_values = Vec::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                if !unique_values.contains(value) {
                    unique_values.push(value.clone());
                }
            }
        }

        Some((unique_values.len(), unique_values))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,department").unwrap();
        writeln!(temp_file, "1,Alice,Engineering").unwrap();
        writeln!(temp_file, "2,Bob,Marketing").unwrap();
        writeln!(temp_file, "3,Charlie,Engineering").unwrap();
        writeln!(temp_file, "4,Diana,Sales").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers(), vec!["id", "name", "department"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let marketing_records = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_records.len(), 1);
    }

    #[test]
    fn test_column_summary() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let summary = processor.get_column_summary("department").unwrap();
        assert_eq!(summary.0, 3);
        assert!(summary.1.contains(&"Engineering".to_string()));
        assert!(summary.1.contains(&"Marketing".to_string()));
        assert!(summary.1.contains(&"Sales".to_string()));
    }
}