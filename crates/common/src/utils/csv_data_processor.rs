use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn read_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            self.headers = first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn write_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(file_path)?;
        writeln!(file, "{}", self.headers.join(","))?;

        for record in &self.records {
            writeln!(file, "{}", record.join(","))?;
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: Vec<String>) -> Result<(), &'static str> {
        if record.len() != self.headers.len() {
            return Err("Record length does not match headers");
        }
        self.records.push(record);
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_data = "name,age,city\nAlice,30,London\nBob,25,Paris";
        let test_file = "test_input.csv";
        fs::write(test_file, test_data).unwrap();

        let mut processor = CsvProcessor::new();
        processor.read_from_file(test_file).unwrap();

        assert_eq!(processor.get_headers(), &vec!["name", "age", "city"]);
        assert_eq!(processor.get_record_count(), 2);

        processor
            .add_record(vec!["Charlie".to_string(), "35".to_string(), "Berlin".to_string()])
            .unwrap();
        assert_eq!(processor.get_record_count(), 3);

        let output_file = "test_output.csv";
        processor.write_to_file(output_file).unwrap();

        let output_content = fs::read_to_string(output_file).unwrap();
        assert!(output_content.contains("Charlie,35,Berlin"));

        fs::remove_file(test_file).unwrap();
        fs::remove_file(output_file).unwrap();
    }
}