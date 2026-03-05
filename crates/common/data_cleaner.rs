
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.insert(normalized.clone()) {
            Some(normalized)
        } else {
            None
        }
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  HELLO  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("hello"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.process_string("  "), None);
        
        assert_eq!(cleaner.get_unique_count(), 1);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "BANANA", "  banana  ", "cherry"];
        
        let result = cleaner.process_batch(&inputs);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }
}
use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;
use std::io;

pub fn filter_numeric_column(input_path: &str, output_path: &str, column_index: usize) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut wtr = Writer::from_path(output_path)?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            if field.parse::<f64>().is_ok() {
                wtr.write_record(&record)?;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_numeric_column() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,25,London").unwrap();
        writeln!(input_file, "Bob,invalid,Paris").unwrap();
        writeln!(input_file, "Charlie,30,Berlin").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        filter_numeric_column(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            1,
        ).unwrap();

        let mut rdr = Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = rdr.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][1], "25");
        assert_eq!(records[1][1], "30");
    }
}