
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

            if cleaned_columns.iter().any(|col| !col.is_empty()) {
                writeln!(output_file, "{}", cleaned_columns.join(","))?;
            }
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

        let test_data = "name,age,city\nJohn,25,NYC\n\n  ,  ,  \nAlice,30,London\n\n";
        fs::write(test_input, test_data).unwrap();

        clean_csv(test_input, test_output).unwrap();

        let result = fs::read_to_string(test_output).unwrap();
        let expected = "name,age,city\nJohn,25,NYC\nAlice,30,London\n";

        assert_eq!(result, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    pub remove_nulls: bool,
    pub trim_whitespace: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_nulls: true,
            trim_whitespace: true,
        }
    }

    pub fn clean_string(&self, input: Option<String>) -> Option<String> {
        match input {
            Some(mut s) => {
                if self.trim_whitespace {
                    s = s.trim().to_string();
                }
                if s.is_empty() && self.remove_nulls {
                    None
                } else {
                    Some(s)
                }
            }
            None => None,
        }
    }

    pub fn clean_hashmap(&self, data: HashMap<String, Option<String>>) -> HashMap<String, Option<String>> {
        data.into_iter()
            .map(|(key, value)| (key, self.clean_string(value)))
            .filter(|(_, value)| value.is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_string(Some("  hello  ".to_string())), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string(Some("   ".to_string())), None);
        assert_eq!(cleaner.clean_string(None), None);
    }

    #[test]
    fn test_clean_hashmap() {
        let cleaner = DataCleaner::new();
        let mut data = HashMap::new();
        data.insert("name".to_string(), Some("  john  ".to_string()));
        data.insert("age".to_string(), Some("25".to_string()));
        data.insert("empty".to_string(), Some("   ".to_string()));

        let cleaned = cleaner.clean_hashmap(data);
        assert_eq!(cleaned.get("name"), Some(&Some("john".to_string())));
        assert_eq!(cleaned.get("empty"), None);
    }
}