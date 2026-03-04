use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

pub fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
        }
        
        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;
        
        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }
    
    Ok(records)
}

pub fn filter_active_records(records: &[CsvRecord]) -> Vec<&CsvRecord> {
    records.iter()
        .filter(|record| record.active)
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter()
        .map(|record| record.value)
        .sum()
}

pub fn find_record_by_id(records: &[CsvRecord], target_id: u32) -> Option<&CsvRecord> {
    records.iter()
        .find(|record| record.id == target_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,true").unwrap();
        
        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 20.0);
        assert!(records[2].active);
    }
    
    #[test]
    fn test_filter_active_records() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].id, 1);
        assert_eq!(active[1].id, 3);
    }
    
    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            CsvRecord { id: 1, name: "Item1".to_string(), value: 5.5, active: true },
            CsvRecord { id: 2, name: "Item2".to_string(), value: 3.5, active: true },
        ];
        
        let total = calculate_total_value(&records);
        assert_eq!(total, 9.0);
    }
    
    #[test]
    fn test_find_record_by_id() {
        let records = vec![
            CsvRecord { id: 101, name: "Alpha".to_string(), value: 50.0, active: true },
            CsvRecord { id: 202, name: "Beta".to_string(), value: 75.0, active: false },
        ];
        
        let found = find_record_by_id(&records, 202);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Beta");
        
        let not_found = find_record_by_id(&records, 999);
        assert!(not_found.is_none());
    }
}