
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvFilter {
    delimiter: char,
    has_header: bool,
}

impl CsvFilter {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvFilter {
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

        let mut filtered_rows = Vec::new();

        for line_result in lines {
            let line = line_result?;
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

    pub fn extract_column(&self, rows: &[Vec<String>], column_index: usize) -> Vec<String> {
        rows.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_and_extract() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let filter = CsvFilter::new(',', true);
        let filtered = filter
            .filter_rows(temp_file.path(), |row| {
                row.get(1)
                    .and_then(|age_str| age_str.parse::<u32>().ok())
                    .map_or(false, |age| age >= 30)
            })
            .unwrap();

        assert_eq!(filtered.len(), 2);
        
        let names = filter.extract_column(&filtered, 0);
        assert!(names.contains(&"Alice".to_string()));
        assert!(names.contains(&"Charlie".to_string()));
    }
}