use std::collections::HashMap;

#[derive(Debug)]
pub struct UserData {
    id: u32,
    name: String,
    email: String,
    age: u8,
}

impl UserData {
    pub fn new(id: u32, name: String, email: String, age: u8) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        if age > 120 {
            return Err("Age must be less than 120".to_string());
        }
        
        Ok(Self { id, name, email, age })
    }
    
    pub fn to_json(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.to_string());
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("age".to_string(), self.age.to_string());
        map
    }
}

pub fn process_user_data(users: Vec<UserData>) -> Vec<HashMap<String, String>> {
    users
        .into_iter()
        .filter(|user| user.age >= 18)
        .map(|user| user.to_json())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_user_creation() {
        let user = UserData::new(1, "John".to_string(), "john@example.com".to_string(), 25);
        assert!(user.is_ok());
    }
    
    #[test]
    fn test_invalid_email() {
        let user = UserData::new(2, "Jane".to_string(), "invalid-email".to_string(), 30);
        assert!(user.is_err());
    }
    
    #[test]
    fn test_process_adult_users() {
        let users = vec![
            UserData::new(1, "Alice".to_string(), "alice@example.com".to_string(), 17).unwrap(),
            UserData::new(2, "Bob".to_string(), "bob@example.com".to_string(), 25).unwrap(),
        ];
        
        let processed = process_user_data(users);
        assert_eq!(processed.len(), 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut records = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let start_index = if self.has_header { 1 } else { 0 };
        
        for line in lines.iter().skip(start_index) {
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }
        
        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    i + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), String> {
        if records.is_empty() {
            return Err("No records available for statistics".to_string());
        }

        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index));
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            }
        }

        if values.is_empty() {
            return Err("No valid numeric values found".to_string());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_records(&records);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.calculate_statistics(&records, 0);
        
        assert!(result.is_ok());
        let (mean, variance, std_dev) = result.unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}