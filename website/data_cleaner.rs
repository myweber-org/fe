use std::collections::HashSet;

pub struct DataCleaner {
    seen_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            seen_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        if self.seen_ids.contains(id) {
            false
        } else {
            self.seen_ids.insert(id.to_string());
            true
        }
    }

    pub fn validate_email(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn normalize_phone(phone: &str) -> Option<String> {
        let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();
        
        match digits.len() {
            10 => Some(format!("+1{}", digits)),
            11 if digits.starts_with('1') => Some(format!("+{}", digits)),
            12 if digits.starts_with("+1") => Some(digits),
            _ => None,
        }
    }

    pub fn clean_text(text: &str) -> String {
        text.trim()
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .replace("\r\n", "\n")
            .replace('\r', "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("user123"));
        assert!(!cleaner.deduplicate("user123"));
        assert!(cleaner.deduplicate("user456"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com"));
        assert!(DataCleaner::validate_email("user.name@domain.co.uk"));
        assert!(!DataCleaner::validate_email("invalid-email"));
        assert!(!DataCleaner::validate_email("@domain.com"));
        assert!(!DataCleaner::validate_email("user@.com"));
    }

    #[test]
    fn test_normalize_phone() {
        assert_eq!(DataCleaner::normalize_phone("555-123-4567"), Some("+15551234567".to_string()));
        assert_eq!(DataCleaner::normalize_phone("1-555-123-4567"), Some("+15551234567".to_string()));
        assert_eq!(DataCleaner::normalize_phone("+1-555-123-4567"), Some("+15551234567".to_string()));
        assert_eq!(DataCleaner::normalize_phone("123"), None);
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(DataCleaner::clean_text("  hello\tworld\n"), "hello\tworld\n");
        assert_eq!(DataCleaner::clean_text("text\r\nwith\r\nlinebreaks"), "text\nwith\nlinebreaks");
    }
}use csv::{Reader, Writer};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;
    
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;
    
    let mut seen = HashSet::new();
    
    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .collect();
        
        let record_key = cleaned_record.join("|");
        
        if !seen.contains(&record_key) {
            seen.insert(record_key);
            writer.write_record(&cleaned_record)?;
        }
    }
    
    writer.flush()?;
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
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "John,25,New York").unwrap();
        writeln!(input_file, " Jane , 30 , Boston ").unwrap();
        writeln!(input_file, "John,25,New York").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let mut reader = Reader::from_path(output_file.path()).unwrap();
        let records: Vec<_> = reader.records().collect();
        assert_eq!(records.len(), 2);
    }
}