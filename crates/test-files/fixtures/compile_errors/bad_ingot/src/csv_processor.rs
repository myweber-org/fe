
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn filter_rows<P, F>(&self, path: P, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut results = Vec::new();

        if self.has_headers {
            let _headers = lines.next().transpose()?;
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn count_matching_rows<P, F>(&self, path: P, predicate: F) -> Result<usize, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: Fn(&[String]) -> bool,
    {
        let matching_rows = self.filter_rows(path, predicate)?;
        Ok(matching_rows.len())
    }
}

pub fn create_default_processor() -> CsvProcessor {
    CsvProcessor::new(',', true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,Tokyo").unwrap();
        file
    }

    #[test]
    fn test_filter_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        
        let result = processor.filter_rows(file.path(), |fields| {
            fields.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        }).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_count_matching_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        
        let count = processor.count_matching_rows(file.path(), |fields| {
            fields.get(2).map_or(false, |city| city.contains("o"))
        }).unwrap();

        assert_eq!(count, 2);
    }
}