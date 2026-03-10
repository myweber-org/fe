
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_header && index == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();

        for (row_index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Row {}: Column index out of bounds", row_index));
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", row_index, record[column_index])),
            }
        }

        Ok(numeric_values)
    }

    pub fn calculate_column_statistics(&self, numeric_values: &[f64]) -> (f64, f64, f64) {
        if numeric_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn transform_column<F>(&self, records: &mut [Vec<String>], column_index: usize, transform_fn: F) -> Result<(), String>
    where
        F: Fn(&str) -> String,
    {
        for (row_index, record) in records.iter_mut().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Row {}: Column index out of bounds", row_index));
            }

            let original = &record[column_index];
            record[column_index] = transform_fn(original);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,55000").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000"]);

        let ages = processor.validate_numeric_column(&records, 1).unwrap();
        assert_eq!(ages, vec![30.0, 25.0, 35.0]);

        let stats = processor.calculate_column_statistics(&ages);
        assert!((stats.0 - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_column_transformation() {
        let mut records = vec![
            vec!["data1".to_string(), "100".to_string()],
            vec!["data2".to_string(), "200".to_string()],
        ];

        let processor = CsvProcessor::new(',', false);
        processor.transform_column(&mut records, 1, |s| format!("${}", s)).unwrap();

        assert_eq!(records[0][1], "$100");
        assert_eq!(records[1][1], "$200");
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records_by_category(
    input_path: &Path,
    output_path: &Path,
    target_category: &str,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.category == target_category && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_category_average(input_path: &Path) -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut category_totals: std::collections::HashMap<String, (f64, u32)> =
        std::collections::HashMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.active {
            let entry = category_totals
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }
    }

    let mut averages: Vec<(String, f64)> = category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect();
    averages.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(averages)
}

fn generate_sample_data(output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;
    let sample_records = vec![
        Record {
            id: 1,
            name: String::from("Item A"),
            category: String::from("Electronics"),
            value: 299.99,
            active: true,
        },
        Record {
            id: 2,
            name: String::from("Item B"),
            category: String::from("Books"),
            value: 24.50,
            active: true,
        },
        Record {
            id: 3,
            name: String::from("Item C"),
            category: String::from("Electronics"),
            value: 599.99,
            active: false,
        },
        Record {
            id: 4,
            name: String::from("Item D"),
            category: String::from("Clothing"),
            value: 49.99,
            active: true,
        },
        Record {
            id: 5,
            name: String::from("Item E"),
            category: String::from("Books"),
            value: 15.99,
            active: true,
        },
    ];

    for record in sample_records {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_path = Path::new("sample_data.csv");
    let filtered_path = Path::new("filtered_electronics.csv");

    generate_sample_data(sample_path)?;
    println!("Generated sample data at: {:?}", sample_path);

    filter_records_by_category(sample_path, filtered_path, "Electronics")?;
    println!("Filtered records saved to: {:?}", filtered_path);

    let averages = calculate_category_average(sample_path)?;
    println!("Category averages:");
    for (category, avg) in averages {
        println!("  {}: {:.2}", category, avg);
    }

    Ok(())
}