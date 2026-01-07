use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct Record {
    id: u32,
    category: String,
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
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn calculate_category_averages(&self) -> HashMap<String, f64> {
        let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            if record.active {
                let entry = category_sums
                    .entry(record.category.clone())
                    .or_insert((0.0, 0));
                entry.0 += record.value;
                entry.1 += 1;
            }
        }

        category_sums
            .into_iter()
            .map(|(category, (sum, count))| (category, sum / count as f64))
            .collect()
    }

    fn find_max_value_record(&self) -> Option<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_file("data.csv") {
        Ok(_) => println!("Data loaded successfully"),
        Err(e) => {
            eprintln!("Failed to load data: {}", e);
            return Ok(());
        }
    }

    let electronics = processor.filter_by_category("Electronics");
    println!("Active Electronics records: {}", electronics.len());

    let averages = processor.calculate_category_averages();
    for (category, avg) in &averages {
        println!("{} average: {:.2}", category, avg);
    }

    if let Some(max_record) = processor.find_max_value_record() {
        println!("Highest value record: ID {}, Value: {}", max_record.id, max_record.value);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let record = Record::from_csv_line("1,Electronics,99.99,true").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.value, 99.99);
        assert!(record.active);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,Electronics,100.0,true").unwrap();
        writeln!(temp_file, "2,Books,50.0,true").unwrap();
        writeln!(temp_file, "3,Electronics,75.0,false").unwrap();
        
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 1);
        assert_eq!(electronics[0].id, 1);
    }

    #[test]
    fn test_calculate_averages() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,Electronics,100.0,true").unwrap();
        writeln!(temp_file, "2,Electronics,200.0,true").unwrap();
        writeln!(temp_file, "3,Books,50.0,true").unwrap();
        
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let averages = processor.calculate_category_averages();
        assert_eq!(averages["Electronics"], 150.0);
        assert_eq!(averages["Books"], 50.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
        let value = parts[2].parse::<f64>()?;
        let category = parts[3].to_string();

        records.push(Record::new(id, name, value, category));
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

    let sum: f64 = records.iter().map(|record| record.value).sum();
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
            Record::new(1, "ItemA".to_string(), 10.5, "Alpha".to_string()),
            Record::new(2, "ItemB".to_string(), 20.3, "Beta".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Alpha".to_string()),
        ];

        let filtered = filter_by_category(&records, "Alpha");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.0, "Test".to_string()),
            Record::new(2, "ItemB".to_string(), 20.0, "Test".to_string()),
            Record::new(3, "ItemC".to_string(), 30.0, "Test".to_string()),
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs).unwrap();
        assert_eq!(avg, 20.0);
    }
}