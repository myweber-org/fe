use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_by_column(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let records = self.process()?;
        let filtered: Vec<Vec<String>> = records
            .into_iter()
            .filter(|record| {
                if column_index < record.len() {
                    record[column_index] == filter_value
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    pub fn get_column_stats(&self, column_index: usize) -> Result<(usize, String, String), Box<dyn Error>> {
        let records = self.process()?;
        let mut values = Vec::new();
        
        for record in &records {
            if column_index < record.len() {
                values.push(record[column_index].clone());
            }
        }

        let count = values.len();
        let min_value = values.iter().min().unwrap_or(&String::new()).clone();
        let max_value = values.iter().max().unwrap_or(&String::new()).clone();

        Ok((count, min_value, max_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,apple,100").unwrap();
        writeln!(temp_file, "2,banana,200").unwrap();
        writeln!(temp_file, "3,apple,150").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 4);
        assert_eq!(result[1][1], "apple");
    }

    #[test]
    fn test_filter_by_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,apple,100").unwrap();
        writeln!(temp_file, "2,banana,200").unwrap();
        writeln!(temp_file, "3,apple,150").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.filter_by_column(1, "apple").unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
    }
}