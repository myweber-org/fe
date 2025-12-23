use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_csv(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 {
                writeln!(output_file, "{}", self.normalize_header(&line))?;
                continue;
            }

            let cleaned_line = self.clean_data_line(&line);
            if !cleaned_line.trim().is_empty() {
                writeln!(output_file, "{}", cleaned_line)?;
            }
        }

        Ok(())
    }

    fn normalize_header(&self, header: &str) -> String {
        header
            .split(',')
            .map(|col| col.trim().to_lowercase().replace(' ', "_"))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn clean_data_line(&self, line: &str) -> String {
        line.split(',')
            .map(|field| {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    "NULL".to_string()
                } else if let Ok(num) = trimmed.parse::<f64>() {
                    format!("{:.2}", num)
                } else {
                    trimmed.replace('"', "").to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cleaner_normalizes_header() {
        let cleaner = DataCleaner::new("dummy.csv", "output.csv");
        let header = "Name, Age, Salary";
        let normalized = cleaner.normalize_header(header);
        assert_eq!(normalized, "name,age,salary");
    }

    #[test]
    fn test_cleaner_handles_empty_fields() {
        let cleaner = DataCleaner::new("dummy.csv", "output.csv");
        let line = "John,,25000.5";
        let cleaned = cleaner.clean_data_line(line);
        assert_eq!(cleaned, "John,NULL,25000.50");
    }
}