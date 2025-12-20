use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub struct CsvMerger {
    output_path: String,
    include_headers: bool,
}

impl CsvMerger {
    pub fn new(output_path: String) -> Self {
        CsvMerger {
            output_path,
            include_headers: true,
        }
    }

    pub fn set_include_headers(&mut self, include: bool) {
        self.include_headers = include;
    }

    pub fn merge_files(&self, input_files: &[String]) -> Result<(), Box<dyn Error>> {
        if input_files.is_empty() {
            return Err("No input files provided".into());
        }

        let output_file = File::create(&self.output_path)?;
        let mut writer = BufWriter::new(output_file);
        let mut first_file = true;

        for (index, file_path) in input_files.iter().enumerate() {
            println!("Processing file {} of {}: {}", index + 1, input_files.len(), file_path);

            let file = File::open(file_path)?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            if let Some(first_line) = lines.next() {
                let header = first_line?;

                if first_file {
                    writer.write_all(header.as_bytes())?;
                    writer.write_all(b"\n")?;
                    first_file = false;
                } else if self.include_headers {
                    writer.write_all(header.as_bytes())?;
                    writer.write_all(b"\n")?;
                }

                for line in lines {
                    let line_content = line?;
                    writer.write_all(line_content.as_bytes())?;
                    writer.write_all(b"\n")?;
                }
            }
        }

        writer.flush()?;
        println!("Successfully merged {} files into {}", input_files.len(), self.output_path);
        Ok(())
    }

    pub fn validate_files(files: &[String]) -> Result<(), Box<dyn Error>> {
        for file in files {
            if !Path::new(file).exists() {
                return Err(format!("File not found: {}", file).into());
            }
            if !file.ends_with(".csv") {
                return Err(format!("File is not a CSV: {}", file).into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_files() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();

        fs::write(&file1, "id,name\n1,Alice\n2,Bob").unwrap();
        fs::write(&file2, "id,name\n3,Charlie\n4,Diana").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let merger = CsvMerger::new(output_file.path().to_str().unwrap().to_string());

        let input_files = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        assert!(merger.merge_files(&input_files).is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Bob"));
        assert!(content.contains("Charlie"));
        assert!(content.contains("Diana"));
    }

    #[test]
    fn test_validate_files() {
        let valid_files = vec!["test.csv".to_string()];
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        assert!(CsvMerger::validate_files(&[temp_path]).is_ok());
        assert!(CsvMerger::validate_files(&valid_files).is_err());
    }
}