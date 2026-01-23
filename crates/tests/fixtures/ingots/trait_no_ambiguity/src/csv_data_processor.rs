use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record {
                id,
                name,
                value,
                category,
            });
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

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|r| r.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    let mut category_totals = std::collections::HashMap::new();

    for record in records {
        let entry = category_totals.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }

    category_totals
        .into_iter()
        .map(|(category, total)| (category, total))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 100.0,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 200.0,
                category: "Electronics".to_string(),
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                value: 150.0,
                category: "Books".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average_value() {
        let records = create_test_records();
        let average = calculate_average_value(&records);
        assert!((average - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = create_test_records();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert!((max_record.value - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = create_test_records();
        let aggregates = aggregate_by_category(&records);
        
        let electronics_total: f64 = aggregates
            .iter()
            .find(|(cat, _)| cat == "Electronics")
            .map(|(_, total)| *total)
            .unwrap_or(0.0);
        
        assert!((electronics_total - 300.0).abs() < 0.001);
    }
}