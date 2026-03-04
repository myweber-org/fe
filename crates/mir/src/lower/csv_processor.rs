use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
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
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<usize, Box<dyn Error>> {
        let filtered = self.filter_rows(file_path, predicate)?;
        Ok(filtered.len())
    }
}

pub fn parse_csv_line(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|field| field.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name,age,city")?;
        writeln!(temp_file, "Alice,30,London")?;
        writeln!(temp_file, "Bob,25,Paris")?;
        writeln!(temp_file, "Charlie,35,London")?;

        let filter = CsvFilter::new(',', true);
        let result = filter.filter_rows(temp_file.path(), |fields| {
            fields.get(2).map(|city| city == "London").unwrap_or(false)
        })?;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
        Ok(())
    }

    #[test]
    fn test_count_matching_rows() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value")?;
        writeln!(temp_file, "1,100")?;
        writeln!(temp_file, "2,200")?;
        writeln!(temp_file, "3,150")?;

        let filter = CsvFilter::new(',', true);
        let count = filter.count_matching_rows(temp_file.path(), |fields| {
            fields
                .get(1)
                .and_then(|v| v.parse::<i32>().ok())
                .map(|n| n > 150)
                .unwrap_or(false)
        })?;

        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn test_parse_csv_line() {
        let line = "apple,banana,cherry";
        let result = parse_csv_line(line, ',');
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    Ok(())
}

fn process_csv(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                wtr.serialize(&record)?;
                valid_count += 1;
            }
            Err(err) => {
                eprintln!("Invalid record {}: {}", record.id, err);
                invalid_count += 1;
            }
        }
    }

    println!("Processed {} records", valid_count + invalid_count);
    println!("Valid records: {}", valid_count);
    println!("Invalid records: {}", invalid_count);

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("input.csv");
    let output_path = Path::new("output.csv");
    
    process_csv(input_path, output_path)
}