
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
            return Err("Invalid number of fields".into());
        }
        
        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;
        
        if name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        
        if value < 0.0 {
            return Err("Value cannot be negative".into());
        }
        
        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }
}

fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }
    
    Ok(records)
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();
    
    (sum, avg, active_count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Processed {} records", records.len());
    
    let (total, average, active_count) = calculate_statistics(&records);
    println!("Total value: {:.2}", total);
    println!("Average value: {:.2}", average);
    println!("Active records: {}", active_count);
    
    if let Some(max_record) = records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap()) {
        println!("Highest value record: ID {}, Name {}", max_record.id, max_record.name);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_parsing() {
        let line = "1,Test Item,42.5,true";
        let record = Record::from_csv_line(line).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.value, 42.5);
        assert_eq!(record.active, true);
    }
    
    #[test]
    fn test_invalid_record() {
        let line = "invalid,Test,not_number,true";
        assert!(Record::from_csv_line(line).is_err());
    }
    
    #[test]
    fn test_empty_name() {
        let line = "1,,42.5,true";
        assert!(Record::from_csv_line(line).is_err());
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (total, avg, active) = calculate_statistics(&records);
        assert_eq!(total, 60.0);
        assert_eq!(avg, 20.0);
        assert_eq!(active, 2);
    }
}