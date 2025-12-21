use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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

    pub fn clean_file<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut cleaned_count = 0;
        let mut lines = reader.lines().enumerate();

        if self.has_headers {
            if let Some((_, header_result)) = lines.next() {
                let header = header_result?;
                writeln!(output_file, "{}", header)?;
            }
        }

        for (line_num, line_result) in lines {
            let line = line_result?;
            let record: Vec<&str> = line.split(self.delimiter).collect();

            if self.is_valid_record(&record) {
                writeln!(output_file, "{}", line)?;
            } else {
                eprintln!("Removing invalid record at line {}", line_num + 1);
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    fn is_valid_record(&self, record: &[&str]) -> bool {
        if record.is_empty() {
            return false;
        }

        for field in record {
            let trimmed = field.trim();
            if trimmed.is_empty() || trimmed.contains('\0') {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_cleaning() {
        let input_data = "name,age,city\nJohn,25,New York\n,30,London\nAlice,,Paris\nBob,35,\0City";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let processor = CsvProcessor::new(',', true);
        let cleaned = processor
            .clean_file(input_file.path(), output_file.path())
            .unwrap();

        assert_eq!(cleaned, 3);

        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        assert_eq!(output_content, "name,age,city\nJohn,25,New York\n");
    }
}