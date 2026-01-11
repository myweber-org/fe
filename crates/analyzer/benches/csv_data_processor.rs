use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[&Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
        ];

        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item A");
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                name: "Test".to_string(),
                value: 10.0,
                category: "Test".to_string(),
            },
            Record {
                id: 2,
                name: "Test".to_string(),
                value: 20.0,
                category: "Test".to_string(),
            },
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs);
        assert_eq!(avg, Some(15.0));
    }
}