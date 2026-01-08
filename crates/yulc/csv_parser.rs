use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvParser {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvParser {
    pub fn from_file<T: AsRef<std::path::Path>>(path: T) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if record.len() == headers.len() {
                records.push(record);
            } else {
                eprintln!("Warning: Skipping malformed record: {}", line);
            }
        }

        Ok(CsvParser { headers, records })
    }

    pub fn get_column<T: FromStr>(&self, column_name: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T::Err: Error + 'static,
    {
        let index = self
            .headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut results = Vec::new();
        for record in &self.records {
            if let Some(value) = record.get(index) {
                match value.parse::<T>() {
                    Ok(parsed) => results.push(parsed),
                    Err(e) => return Err(Box::new(e)),
                }
            }
        }

        Ok(results)
    }

    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();

        let parser = CsvParser::from_file(temp_file.path()).unwrap();
        assert_eq!(parser.row_count(), 2);
        assert_eq!(parser.column_count(), 3);

        let ages: Vec<u32> = parser.get_column("age").unwrap();
        assert_eq!(ages, vec![30, 25]);

        let names: Vec<String> = parser.get_column("name").unwrap();
        assert_eq!(names, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn test_missing_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name").unwrap();
        writeln!(temp_file, "1,Alice").unwrap();

        let parser = CsvParser::from_file(temp_file.path()).unwrap();
        let result: Result<Vec<u32>, _> = parser.get_column("nonexistent");
        assert!(result.is_err());
    }
}