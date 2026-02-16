use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn remove_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect()
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let normalized = normalize_strings(strings);
    let non_empty = remove_empty_strings(normalized);
    deduplicate(non_empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 1, 4];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_remove_empty_strings() {
        let input = vec!["hello".to_string(), "".to_string(), "  ".to_string(), "world".to_string()];
        let result = remove_empty_strings(input);
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_clean_string_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "  Banana  ".to_string(),
            "  ".to_string(),
            "banana".to_string(),
        ];
        let result = clean_string_data(input);
        assert_eq!(result, vec!["apple".to_string(), "banana".to_string()]);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                let trimmed = field.trim();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
                    String::new()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        
        csv_writer.write_record(&cleaned_record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city\n").unwrap();
        writeln!(input_file, "John, 25 ,New York\n").unwrap();
        writeln!(input_file, "Alice, ,London\n").unwrap();
        writeln!(input_file, ",30,Paris\n").unwrap();
        writeln!(input_file, "Bob,35,null\n").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();
        
        let mut rdr = csv::Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = rdr.records().collect();
        
        assert_eq!(records.len(), 4);
        let first_record = &records[0].as_ref().unwrap();
        assert_eq!(first_record[0], "John");
        assert_eq!(first_record[1], "25");
        assert_eq!(first_record[2], "New York");
        
        let second_record = &records[1].as_ref().unwrap();
        assert_eq!(second_record[1], "");
    }
}