use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            category: parts[3].to_string(),
        })
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

        match Record::from_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", index + 1, e),
        }
    }

    Ok(records)
}

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;

    let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = aggregates
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    aggregates
        .into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Total records processed: {}", records.len());
    
    let aggregates = aggregate_by_category(&records);
    
    for (category, total, count) in aggregates {
        println!("Category: {}, Total Value: {:.2}, Record Count: {}", 
                 category, total, count);
    }
    
    let avg_value: f64 = records.iter().map(|r| r.value).sum::<f64>() / records.len() as f64;
    println!("Average value across all records: {:.2}", avg_value);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let line = "1,ProductA,25.50,Electronics";
        let record = Record::from_line(line).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "ProductA");
        assert_eq!(record.value, 25.50);
        assert_eq!(record.category, "Electronics");
    }

    #[test]
    fn test_aggregation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Y".to_string() },
        ];
        
        let aggregates = aggregate_by_category(&records);
        
        assert_eq!(aggregates.len(), 2);
        
        let x_agg = aggregates.iter().find(|(cat, _, _)| cat == "X").unwrap();
        assert_eq!(x_agg.1, 30.0);
        assert_eq!(x_agg.2, 2);
    }
}