use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }
}

fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_records: Vec<&Record> = records.iter().filter(|r| r.active).collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let avg = sum / active_records.len() as f64;
    let max = active_records.iter().map(|r| r.value).fold(0.0, f64::max);

    (avg, max, active_records.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Total records loaded: {}", records.len());
    
    let (average, maximum, active_count) = calculate_statistics(&records);
    println!("Active records: {}", active_count);
    println!("Average value: {:.2}", average);
    println!("Maximum value: {:.2}", maximum);

    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(cell) = parts.get(self.filter_column) {
                if cell.trim() == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column<F>(&self, transform_fn: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() > self.filter_column {
                parts[self.filter_column] = &transform_fn(parts[self.filter_column]);
            }

            let transformed_line = parts.join(",");
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_filtering() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        let content = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Carol,active\n";
        
        fs::write(test_input, content).unwrap();
        
        let processor = CsvProcessor::new(test_input, test_output, 2, "active");
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("Alice"));
        assert!(!output_content.contains("Bob"));
        assert!(output_content.contains("Carol"));
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}