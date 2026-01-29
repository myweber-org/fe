
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn filter_and_transform_records(
    input_path: &Path,
    output_path: &Path,
    min_value: f64,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let transformed = Record {
                name: record.name.to_uppercase(),
                value: record.value * 1.1,
                ..record
            };
            writer.serialize(transformed)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn validate_csv_structure(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let headers = reader.headers()?;
    
    let expected = vec!["id", "name", "value", "active"];
    if headers != expected {
        return Err(format!("Invalid CSV structure. Expected {:?}, got {:?}", expected, headers).into());
    }
    
    let mut record_count = 0;
    for result in reader.deserialize::<Record>() {
        let _ = result?;
        record_count += 1;
    }
    
    println!("Validated {} records in {}", record_count, path.display());
    Ok(())
}

fn process_csv_files() -> Result<(), Box<dyn Error>> {
    let input_file = Path::new("data/input.csv");
    let output_file = Path::new("data/output.csv");
    
    validate_csv_structure(input_file)?;
    filter_and_transform_records(input_file, output_file, 50.0)?;
    validate_csv_structure(output_file)?;
    
    println!("Processing completed successfully");
    Ok(())
}

fn main() {
    if let Err(e) = process_csv_files() {
        eprintln!("Error processing CSV files: {}", e);
        std::process::exit(1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    selected_columns: Vec<usize>,
}

impl CsvProcessor {
    pub fn new(delimiter: char, selected_columns: Vec<usize>) -> Self {
        CsvProcessor {
            delimiter,
            selected_columns,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(self.delimiter).map(String::from).collect();
            
            let filtered_columns: Vec<String> = self.selected_columns
                .iter()
                .filter_map(|&index| columns.get(index).cloned())
                .collect();

            if !filtered_columns.is_empty() {
                results.push(filtered_columns);
            }
        }

        Ok(results)
    }

    pub fn filter_by_column_value(&self, data: &[Vec<String>], column_index: usize, expected_value: &str) -> Vec<Vec<String>> {
        data.iter()
            .filter(|row| row.get(column_index).map(|val| val == expected_value).unwrap_or(false))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', vec![0, 2]);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "city"]);
        assert_eq!(result[1], vec!["Alice", "New York"]);
    }

    #[test]
    fn test_filter_function() {
        let data = vec![
            vec!["apple".to_string(), "red".to_string()],
            vec!["banana".to_string(), "yellow".to_string()],
            vec!["apple".to_string(), "green".to_string()],
        ];

        let processor = CsvProcessor::new(',', vec![0, 1]);
        let filtered = processor.filter_by_column_value(&data, 0, "apple");

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|row| row[0] == "apple"));
    }
}