
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    
    let mut input_data = String::new();
    for line in stdin.lock().lines() {
        input_data.push_str(&line?);
        input_data.push('\n');
    }
    
    let cleaned = clean_data(&input_data);
    write!(output, "{}", cleaned)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\nbanana\napple\n";
        let expected = "apple\nbanana\ncherry\n";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    let mut valid_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= 0.0 && !record.name.is_empty() {
            valid_records.push(record);
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(output_file);

    for record in valid_records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    println!("Cleaned {} valid records", valid_records.len());
    Ok(())
}

fn main() {
    if let Err(err) = clean_csv("input.csv", "output.csv") {
        eprintln!("Error cleaning data: {}", err);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|field| field.trim().to_string())
                .collect();

            writeln!(output_file, "{}", cleaned_columns.join(","))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let mut input_file = File::create(test_input).unwrap();
        writeln!(input_file, "  a , b , c  ").unwrap();
        writeln!(input_file, "").unwrap();
        writeln!(input_file, "x,y,z").unwrap();
        drop(input_file);

        clean_csv(test_input, test_output).unwrap();

        let mut output_file = File::open(test_output).unwrap();
        let mut content = String::new();
        output_file.read_to_string(&mut content).unwrap();

        assert_eq!(content, "a,b,c\nx,y,z\n");

        std::fs::remove_file(test_input).unwrap();
        std::fs::remove_file(test_output).unwrap();
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    duplicates: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            duplicates: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return false;
        }

        if self.duplicates.contains(&trimmed) {
            return false;
        }

        self.duplicates.insert(trimmed.clone());
        self.records.push(trimmed);
        true
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.is_empty()
                    && record.len() <= 255
                    && record.chars().all(|c| c.is_ascii() || c.is_alphanumeric())
            })
            .collect()
    }

    pub fn get_unique_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let unique_records: Vec<String> = self.records.drain(..).collect();
        let mut seen = HashSet::new();
        let mut removed_count = 0;

        for record in unique_records {
            if seen.insert(record.clone()) {
                self.records.push(record);
            } else {
                removed_count += 1;
            }
        }

        self.duplicates = seen;
        removed_count
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.duplicates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test"));
        assert!(!cleaner.add_record("test"));
        assert!(!cleaner.add_record(""));
        assert!(!cleaner.add_record("   "));
    }

    #[test]
    fn test_validate_records() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid123");
        cleaner.add_record("another");
        
        let validation = cleaner.validate_records();
        assert_eq!(validation.len(), 2);
        assert!(validation[0]);
        assert!(validation[1]);
    }

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("duplicate");
        cleaner.add_record("duplicate");
        cleaner.add_record("unique");
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_unique_records().len(), 2);
    }
}