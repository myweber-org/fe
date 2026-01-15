
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    
    let mut input_data = String::new();
    for line in stdin.lock().lines() {
        input_data.push_str(&line?);
        input_data.push('\n');
    }
    
    let cleaned = clean_data(&input_data);
    write!(output, "{}", cleaned)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\nbanana\napple\n";
        let expected = "apple\nbanana\ncherry\n";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}use csv::Reader;
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

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    let mut valid_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= 0.0 && !record.name.is_empty() {
            valid_records.push(record);
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(output_file);

    for record in valid_records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    println!("Cleaned {} valid records", valid_records.len());
    Ok(())
}

fn main() {
    if let Err(err) = clean_csv("input.csv", "output.csv") {
        eprintln!("Error cleaning data: {}", err);
    }
}