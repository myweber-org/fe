use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
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
        if self.input_files.is_empty() {
            return Err("No input files provided".into());
        }

        let output_path = Path::new(&self.output_file);
        let output_file = File::create(output_path)?;
        let mut writer = BufWriter::new(output_file);

        let mut first_file = true;

        for (file_index, input_file) in self.input_files.iter().enumerate() {
            let path = Path::new(input_file);
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            if let Some(first_line) = lines.next() {
                let header = first_line?;

                if first_file {
                    if self.include_headers {
                        writeln!(writer, "{}", header)?;
                    }
                    first_file = false;
                } else if !self.include_headers {
                } else {
                    if file_index == 1 && self.include_headers {
                        writeln!(writer, "{}", header)?;
                    }
                }

                for line in lines {
                    let line = line?;
                    if !line.trim().is_empty() {
                        writeln!(writer, "{}", line)?;
                    }
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}

pub fn validate_csv_file(file_path: &str) -> Result<bool, Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path).into());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines().take(5) {
        let line = line?;
        if line.contains(',') || line.contains(';') || line.contains('\t') {
            return Ok(true);
        }
    }

    Ok(false)
}