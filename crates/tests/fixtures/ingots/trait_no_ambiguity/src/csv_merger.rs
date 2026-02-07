use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvMerger {
    input_files: Vec<String>,
    output_file: String,
    delimiter: char,
    include_headers: bool,
}

impl CsvMerger {
    pub fn new(input_files: Vec<String>, output_file: String) -> Self {
        CsvMerger {
            input_files,
            output_file,
            delimiter: ',',
            include_headers: true,
        }
    }

    pub fn set_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn set_include_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }

    pub fn merge(&self) -> Result<(), Box<dyn Error>> {
        let output_path = Path::new(&self.output_file);
        let mut output = File::create(output_path)?;
        let mut is_first_file = true;

        for input_file in &self.input_files {
            let input_path = Path::new(input_file);
            let file = File::open(input_path)?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            if let Some(first_line) = lines.next() {
                let header = first_line?;

                if is_first_file {
                    if self.include_headers {
                        writeln!(output, "{}", header)?;
                    }
                    is_first_file = false;
                } else if !self.include_headers {
                } else {
                }

                for line in lines {
                    let line_content = line?;
                    if !line_content.trim().is_empty() {
                        writeln!(output, "{}", line_content)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn validate_csv_file(path: &str) -> Result<bool, Box<dyn Error>> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err("File does not exist".into());
    }

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines().take(5) {
        let line_content = line?;
        if line_content.contains(',') || line_content.contains(';') || line_content.contains('\t') {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_merger() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(&file1, "id,name,age\n1,Alice,30\n2,Bob,25").unwrap();
        fs::write(&file2, "id,name,age\n3,Charlie,35\n4,Diana,28").unwrap();

        let merger = CsvMerger::new(
            vec![file1.path().to_str().unwrap().to_string(), 
                 file2.path().to_str().unwrap().to_string()],
            output_file.path().to_str().unwrap().to_string(),
        );

        let result = merger.merge();
        assert!(result.is_ok());

        let content = fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("Diana"));
    }

    #[test]
    fn test_validate_csv_file() {
        let valid_file = NamedTempFile::new().unwrap();
        fs::write(&valid_file, "id,name\n1,test").unwrap();

        let result = validate_csv_file(valid_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}