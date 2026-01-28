
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
        let mut filtered_records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if fields.len() > column_index && fields[column_index] == filter_value {
                filtered_records.push(fields);
            }
        }

        Ok(filtered_records)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count)
    }

    pub fn get_column_values(&self, column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut column_values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if fields.len() > column_index {
                column_values.push(fields[column_index].clone());
            }
        }

        Ok(column_values)
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
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,alice,30").unwrap();
        writeln!(temp_file, "2,bob,25").unwrap();
        writeln!(temp_file, "3,alice,35").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.filter_records(1, "alice").unwrap();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "1");
        assert_eq!(filtered[1][0], "3");
    }

    #[test]
    fn test_count_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let count = processor.count_records().unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_column_values() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "a,b,c").unwrap();
        writeln!(temp_file, "1,2,3").unwrap();
        writeln!(temp_file, "4,5,6").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let column_values = processor.get_column_values(1).unwrap();

        assert_eq!(column_values, vec!["b", "2", "5"]);
    }
}