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

    pub fn process_with_filter<F>(&self, filter_fn: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_fn(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        
        let filtered = processor
            .process_with_filter(|fields| fields.get(1).and_then(|a| a.parse::<i32>().ok()).unwrap_or(0) > 30)
            .unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
    }
}