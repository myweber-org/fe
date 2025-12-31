use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvFilter {
    column_index: usize,
    filter_value: String,
}

impl CsvFilter {
    pub fn new(column_index: usize, filter_value: &str) -> Self {
        CsvFilter {
            column_index,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if let Some(value) = columns.get(self.column_index) {
                if value == &self.filter_value {
                    filtered_rows.push(columns);
                }
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_matches<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let filtered = self.process_file(file_path)?;
        Ok(filtered.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_matching_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,30,New York").unwrap();

        let filter = CsvFilter::new(1, "30");
        let result = filter.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_matches() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,status,value").unwrap();
        writeln!(temp_file, "1,active,100").unwrap();
        writeln!(temp_file, "2,inactive,200").unwrap();
        writeln!(temp_file, "3,active,300").unwrap();

        let filter = CsvFilter::new(1, "active");
        let count = filter.count_matches(temp_file.path()).unwrap();
        
        assert_eq!(count, 2);
    }
}