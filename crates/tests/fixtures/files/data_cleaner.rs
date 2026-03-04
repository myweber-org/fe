use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn normalize_whitespace(&mut self) {
        for record in &mut self.records {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn to_lowercase(&mut self) {
        for record in &mut self.records {
            *record = record.to_lowercase();
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());

        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert!(deduped.contains(&"test".to_string()));
        assert!(deduped.contains(&"unique".to_string()));
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   ".to_string());
        cleaner.normalize_whitespace();

        assert_eq!(cleaner.get_records()[0], "multiple spaces");
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_values: Vec<String>,
    pub normalization_map: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        let mut normalization_map = HashMap::new();
        normalization_map.insert("USA".to_string(), "United States".to_string());
        normalization_map.insert("UK".to_string(), "United Kingdom".to_string());
        normalization_map.insert("UAE".to_string(), "United Arab Emirates".to_string());

        DataCleaner {
            null_values: vec!["null".to_string(), "NULL".to_string(), "".to_string(), "N/A".to_string()],
            normalization_map,
        }
    }

    pub fn clean_string(&self, input: &str) -> Option<String> {
        if self.null_values.contains(&input.to_string()) {
            return None;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        match self.normalization_map.get(trimmed) {
            Some(normalized) => Some(normalized.clone()),
            None => Some(trimmed.to_string()),
        }
    }

    pub fn clean_vector(&self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter_map(|&item| self.clean_string(item))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_string("USA"), Some("United States".to_string()));
        assert_eq!(cleaner.clean_string("null"), None);
        assert_eq!(cleaner.clean_string("   "), None);
        assert_eq!(cleaner.clean_string("valid data"), Some("valid data".to_string()));
    }

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec!["USA", "null", "valid", "", "UK"];
        let cleaned = cleaner.clean_vector(data);
        
        assert_eq!(cleaned, vec!["United States", "valid", "United Kingdom"]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder, Trim};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    
    let mut csv_reader = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record.iter()
            .map(|field| {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    "N/A".to_string()
                } else {
                    trimmed.to_lowercase()
                }
            })
            .collect();
        
        csv_writer.write_record(&cleaned_record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "Name,Age,City\nJohn Doe, 25 , New York\nJane, ,London\n")?;
        
        let mut output_file = NamedTempFile::new()?;
        
        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap())?;
        
        let content = std::fs::read_to_string(output_file.path())?;
        assert!(content.contains("john doe,25,new york"));
        assert!(content.contains("jane,N/A,london"));
        
        Ok(())
    }
}use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            writeln!(output_file, "{}", cleaned_columns.join(","))?;
        }
    }

    Ok(())
}