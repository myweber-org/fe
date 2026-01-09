use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("data.csv")?;
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > filter_column {
                if columns[filter_column] == filter_value {
                    results.push(columns);
                }
            }
        }

        Ok(results)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut sum = 0.0;
        let mut count = 0;
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > column_index {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,apple,10.5").unwrap();
        writeln!(temp_file, "2,banana,15.3").unwrap();
        writeln!(temp_file, "3,apple,8.7").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(1, "apple").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "1");
        assert_eq!(result[1][0], "3");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,price").unwrap();
        writeln!(temp_file, "1,10.0").unwrap();
        writeln!(temp_file, "2,20.0").unwrap();
        writeln!(temp_file, "3,30.0").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert_eq!(average, 20.0);
    }
}