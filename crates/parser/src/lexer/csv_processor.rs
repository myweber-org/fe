use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut filtered = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                filtered.push(fields);
            }
        }

        Ok(filtered)
    }

    pub fn count_matching_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
        expected_value: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let filtered = self.filter_rows(file_path, |fields| {
            fields.get(column_index).map_or(false, |val| val == expected_value)
        })?;
        Ok(filtered.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,30,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor
            .filter_rows(temp_file.path(), |fields| fields[1] == "30")
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_matching_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "product,quantity,price").unwrap();
        writeln!(temp_file, "Apple,100,1.50").unwrap();
        writeln!(temp_file, "Banana,150,0.75").unwrap();
        writeln!(temp_file, "Apple,200,1.50").unwrap();

        let processor = CsvProcessor::new(',', true);
        let count = processor
            .count_matching_rows(temp_file.path(), 0, "Apple")
            .unwrap();

        assert_eq!(count, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&columns) {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
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
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor
            .filter_rows(|cols| cols.get(1).and_then(|a| a.parse::<i32>().ok()).unwrap_or(0) > 30)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_count_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        assert_eq!(processor.count_rows().unwrap(), 3);
    }
}