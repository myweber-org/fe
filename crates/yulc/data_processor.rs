use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_data(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= threshold && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

fn filter_records(records: Vec<Record>, predicate: impl Fn(&Record) -> bool) -> Vec<Record> {
    records.into_iter()
        .filter(predicate)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_filter_function() {
        let records = vec![
            Record { id: 1, name: "Alpha".to_string(), value: 15.5, active: true },
            Record { id: 2, name: "Beta".to_string(), value: 25.0, active: false },
            Record { id: 3, name: "Gamma".to_string(), value: 18.3, active: true },
        ];
        
        let filtered = filter_records(records, |r| r.active && r.value > 16.0);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Gamma");
    }
}