use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();
        
        if filtered_record.len() == headers.len() {
            csv_writer.write_record(&filtered_record)?;
        }
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "name,age,city\nJohn,25,NYC\nJane,,London\nBob,30,\nAlice,28,Boston";
        fs::write(test_input, test_data).unwrap();
        
        clean_csv(test_input, test_output).unwrap();
        
        let cleaned = fs::read_to_string(test_output).unwrap();
        assert_eq!(cleaned, "name,age,city\nJohn,25,NYC\nAlice,28,Boston\n");
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub records: Vec<String>,
    pub deduplicated: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            deduplicated: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        self.deduplicated.clear();
        let mut unique_records = Vec::new();

        for record in &self.records {
            if self.deduplicated.insert(record.clone()) {
                unique_records.push(record.clone());
            }
        }

        unique_records
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty()
                    && record.len() <= 1000
                    && !record.contains("NULL")
                    && !record.contains("undefined")
            })
            .collect()
    }

    pub fn clean_all(&mut self) -> (Vec<String>, Vec<bool>) {
        let deduplicated = self.deduplicate();
        let validation_results = self.validate_records();
        (deduplicated, validation_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("record1");
        cleaner.add_record("record2");
        cleaner.add_record("record1");
        
        let result = cleaner.deduplicate();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"record1".to_string()));
        assert!(result.contains(&"record2".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid record");
        cleaner.add_record("");
        cleaner.add_record("record with NULL value");
        
        let results = cleaner.validate_records();
        assert_eq!(results, vec![true, false, false]);
    }
}