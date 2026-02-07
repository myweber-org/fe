use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
struct Statistics {
    count: usize,
    total_value: f64,
    average_value: f64,
    categories: Vec<String>,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            count: 0,
            total_value: 0.0,
            average_value: 0.0,
            categories: Vec::new(),
        }
    }
}

fn process_csv_file(input_path: &str, output_path: &str) -> Result<Statistics, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(input_file);
    
    let mut stats = Statistics::new();
    let mut records: Vec<Record> = Vec::new();
    let mut unique_categories = std::collections::HashSet::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        stats.count += 1;
        stats.total_value += record.value;
        unique_categories.insert(record.category.clone());
        
        records.push(record);
    }

    if stats.count > 0 {
        stats.average_value = stats.total_value / stats.count as f64;
    }
    
    stats.categories = unique_categories.into_iter().collect();
    stats.categories.sort();

    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(stats)
}

fn filter_records_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .collect()
}

fn calculate_category_stats(records: &[Record], category: &str) -> (f64, f64, usize) {
    let filtered: Vec<&Record> = filter_records_by_category(records, category);
    
    let count = filtered.len();
    let total: f64 = filtered.iter().map(|r| r.value).sum();
    let average = if count > 0 { total / count as f64 } else { 0.0 };
    
    (total, average, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), value: 10.5, category: "Alpha".to_string() },
            Record { id: 2, name: "Item B".to_string(), value: 20.0, category: "Beta".to_string() },
            Record { id: 3, name: "Item C".to_string(), value: 15.5, category: "Alpha".to_string() },
        ];

        let mut stats = Statistics::new();
        for record in &records {
            stats.count += 1;
            stats.total_value += record.value;
        }
        stats.average_value = stats.total_value / stats.count as f64;

        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_value, 46.0);
        assert_eq!(stats.average_value, 46.0 / 3.0);
    }

    #[test]
    fn test_category_filtering() {
        let records = vec![
            Record { id: 1, name: "Test 1".to_string(), value: 5.0, category: "X".to_string() },
            Record { id: 2, name: "Test 2".to_string(), value: 10.0, category: "Y".to_string() },
            Record { id: 3, name: "Test 3".to_string(), value: 15.0, category: "X".to_string() },
        ];

        let filtered = filter_records_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X"));
    }
}