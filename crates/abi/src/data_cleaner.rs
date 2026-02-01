use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            writeln!(output_file, "{}", cleaned_columns.join(","))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let input_content = "  col1, col2 , col3  \n\n,,\nvalid,data,here  ";
        fs::write(test_input, input_content).unwrap();

        clean_csv(test_input, test_output).unwrap();

        let output_content = fs::read_to_string(test_output).unwrap();
        assert_eq!(output_content, "col1,col2,col3\nvalid,data,here\n");

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.data {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.data
    }

    pub fn process(&mut self) {
        self.normalize_strings();
        self.remove_duplicates();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let mut cleaner = DataCleaner::new(vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ]);
        
        cleaner.process();
        let result = cleaner.get_data();
        
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new(vec![
            "  Apple  ".to_string(),
            "BANANA".to_string(),
            "  Cherry  ".to_string(),
        ]);
        
        cleaner.normalize_strings();
        let result = cleaner.get_data();
        
        assert_eq!(result[0], "apple");
        assert_eq!(result[1], "banana");
        assert_eq!(result[2], "cherry");
    }
}