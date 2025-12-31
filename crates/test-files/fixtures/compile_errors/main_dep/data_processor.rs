use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn filter_records(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if column_index < fields.len() && fields[column_index] == filter_value {
                filtered.push(fields);
            }
        }

        Ok(filtered)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }

    pub fn get_column_values(&self, column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if column_index < fields.len() {
                values.push(fields[column_index].clone());
            }
        }

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age\n1,alice,30\n2,bob,25\n3,alice,28").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.filter_records(1, "alice").unwrap();
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
    }

    #[test]
    fn test_count_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "a,b,c\n1,2,3\n4,5,6").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        assert_eq!(processor.count_records().unwrap(), 3);
    }
}