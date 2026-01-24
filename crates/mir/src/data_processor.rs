use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            eprintln!("Warning: Line {} has invalid format", line_num + 1);
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid ID on line {}", line_num + 1);
                continue;
            }
        };

        let value = match parts[1].parse::<f64>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid value on line {}", line_num + 1);
                continue;
            }
        };

        let record = DataRecord::new(id, value, parts[2].to_string());
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid data on line {}", line_num + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "category_a".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "category_a".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "category_a".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "cat1".to_string()),
            DataRecord::new(2, 20.0, "cat2".to_string()),
            DataRecord::new(3, 30.0, "cat3".to_string()),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}