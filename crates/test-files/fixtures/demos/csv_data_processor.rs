
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

pub fn load_records_from_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
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

        records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    Ok(records)
}

pub fn filter_records_by_category(records: Vec<Record>, category_filter: &str) -> Vec<Record> {
    records
        .into_iter()
        .filter(|record| record.category == category_filter)
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|record| record.value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_and_filter_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,30.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,Electronics").unwrap();

        let records = load_records_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 3);

        let filtered = filter_records_by_category(records, "Electronics");
        assert_eq!(filtered.len(), 2);

        let total = calculate_total_value(&filtered);
        assert_eq!(total, 41.25);
    }
}