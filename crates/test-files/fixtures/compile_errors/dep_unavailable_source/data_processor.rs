use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
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

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let numeric_values: Vec<f64> = data
            .iter()
            .filter_map(|record| record.get(column_index).and_then(|s| s.parse::<f64>().ok()))
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        assert!(processor.validate_record(&result[0]));

        let stats = processor.calculate_statistics(&result, 2).unwrap();
        assert!((stats.0 - 50000.0).abs() < 0.1);
    }

    #[test]
    fn test_invalid_data() {
        let processor = DataProcessor::new(',', false);
        let empty_record = vec![];
        assert!(!processor.validate_record(&empty_record));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => continue,
            };

            let record = Record {
                id,
                name,
                value,
                active,
            };

            self.records.push(record);
            loaded_count += 1;
        }

        Ok(loaded_count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,20.0,false").unwrap();
        writeln!(temp_file, "3,Test3,15.75,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(Record {
            id: 1,
            name: "Test1".to_string(),
            value: 10.5,
            active: true,
        });
        
        processor.records.push(Record {
            id: 2,
            name: "Test2".to_string(),
            value: 20.0,
            active: false,
        });
        
        processor.records.push(Record {
            id: 3,
            name: "Test3".to_string(),
            value: 15.75,
            active: true,
        });

        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 2);
        assert!(active_records.iter().all(|r| r.active));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(Record {
            id: 1,
            name: "Test1".to_string(),
            value: 10.0,
            active: true,
        });
        
        processor.records.push(Record {
            id: 2,
            name: "Test2".to_string(),
            value: 20.0,
            active: true,
        });
        
        processor.records.push(Record {
            id: 3,
            name: "Test3".to_string(),
            value: 30.0,
            active: true,
        });

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_find_by_id() {
        let mut processor = DataProcessor::new();
        
        let target_record = Record {
            id: 42,
            name: "Target".to_string(),
            value: 99.9,
            active: true,
        };
        
        processor.records.push(Record {
            id: 1,
            name: "Test1".to_string(),
            value: 10.0,
            active: true,
        });
        
        processor.records.push(target_record.clone());
        
        processor.records.push(Record {
            id: 3,
            name: "Test3".to_string(),
            value: 30.0,
            active: true,
        });

        let found = processor.find_by_id(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, target_record.id);
        assert_eq!(found.unwrap().name, target_record.name);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
        assert_eq!(processor.filter_active().len(), 0);
    }
}