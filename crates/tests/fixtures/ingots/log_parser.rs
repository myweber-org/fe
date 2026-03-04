
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut errors = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.to_lowercase().contains("error") {
                errors.push(line);
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self) -> io::Result<usize> {
        let errors = self.extract_errors()?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: System started").unwrap();
        writeln!(temp_file, "ERROR: Disk full").unwrap();
        writeln!(temp_file, "WARNING: High memory usage").unwrap();
        writeln!(temp_file, "ERROR: Network timeout").unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Disk full"));
        assert!(errors[1].contains("Network timeout"));
    }

    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Operation completed").unwrap();
        writeln!(temp_file, "ERROR: Permission denied").unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let count = parser.count_errors().unwrap();

        assert_eq!(count, 1);
    }
}