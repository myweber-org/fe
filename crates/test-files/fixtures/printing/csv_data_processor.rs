use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) if record.is_valid() => records.push(record),
            Ok(_) => eprintln!("Skipping invalid record at line {}", index + 1),
            Err(e) => eprintln!("Error parsing line {}: {}", index + 1, e),
        }
    }

    Ok(records)
}

fn filter_records(records: &[Record], min_value: f64) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.value >= min_value && r.active)
        .collect()
}

fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data.csv";
    let records = process_csv_file(file_path)?;

    println!("Total valid records: {}", records.len());

    let filtered = filter_records(&records, 50.0);
    println!("Filtered records (value >= 50.0 and active): {}", filtered.len());

    if let Some(avg) = calculate_average(&filtered) {
        println!("Average value of filtered records: {:.2}", avg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record::from_csv_line("1,Test,100.5,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 100.5);
        assert!(record.active);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 10.0,
            active: true,
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 30.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 60.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 80.0, active: false },
        ];

        let filtered = filter_records(&records, 50.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
    }
}