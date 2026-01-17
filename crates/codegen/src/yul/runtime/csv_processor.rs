use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn read_file(&self, path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            records.push(fields);
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn transform_column<F>(&self, records: &mut [Vec<String>], col_index: usize, transformer: F)
    where
        F: Fn(&str) -> String,
    {
        for record in records.iter_mut() {
            if col_index < record.len() {
                record[col_index] = transformer(&record[col_index]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);

        let filtered = processor.filter_records(&records, |fields| {
            fields.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Tokyo"]);

        let mut transformed_records = records.clone();
        processor.transform_column(&mut transformed_records, 2, |city| city.to_uppercase());

        assert_eq!(transformed_records[0][2], "NEW YORK");
    }
}