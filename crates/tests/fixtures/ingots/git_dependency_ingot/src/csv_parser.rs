
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
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
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
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            } else {
                return Err(format!(
                    "Record length mismatch: expected {}, got {}",
                    headers.len(),
                    record.len()
                ).into());
            }
        }

        Ok(CsvParser { headers, records })
    }

    pub fn get_column<T: FromStr>(&self, column_name: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T::Err: Error + 'static,
    {
        let index = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut result = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let value = record.get(index)
                .ok_or("Record missing column")?;
            let parsed = value.parse::<T>()?;
            result.push(parsed);
        }

        Ok(result)
    }

    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(parser.column_count(), 3);
        assert_eq!(parser.row_count(), 2);

        let ages: Vec<i32> = parser.get_column("age").unwrap();
        assert_eq!(ages, vec![30, 25]);
    }
}