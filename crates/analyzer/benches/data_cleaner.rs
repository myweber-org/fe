use csv::{Reader, Writer};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RawRecord {
    id: String,
    value: String,
    category: String,
    active: String,
}

#[derive(Debug)]
struct CleanRecord {
    id: u32,
    value: f64,
    category: String,
    active: bool,
}

impl TryFrom<RawRecord> for CleanRecord {
    type Error = String;

    fn try_from(raw: RawRecord) -> Result<Self, Self::Error> {
        let id = raw.id.parse().map_err(|e| format!("Invalid ID: {}", e))?;
        let value = raw.value.parse().map_err(|e| format!("Invalid value: {}", e))?;
        let active = match raw.active.to_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => return Err(format!("Invalid active flag: {}", raw.active)),
        };

        Ok(CleanRecord {
            id,
            value,
            category: raw.category,
            active,
        })
    }
}

fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(input_file);
    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);

    for result in rdr.deserialize() {
        let raw_record: RawRecord = result?;
        
        match CleanRecord::try_from(raw_record) {
            Ok(clean_record) => {
                if clean_record.active && clean_record.value > 0.0 {
                    wtr.serialize(&clean_record)?;
                }
            }
            Err(e) => eprintln!("Skipping record: {}", e),
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("input.csv");
    let output_path = Path::new("cleaned_output.csv");
    
    clean_csv_data(input_path, output_path)?;
    println!("Data cleaning completed successfully");
    Ok(())
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub items: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    pub fn remove_duplicates(&mut self) {
        let mut seen = HashSet::new();
        self.items.retain(|item| seen.insert(item.clone()));
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.items {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn clean(&mut self) {
        self.normalize_strings();
        self.remove_duplicates();
        self.items.sort();
    }

    pub fn get_results(&self) -> &Vec<String> {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("  Apple ");
        cleaner.add_item("banana");
        cleaner.add_item("  apple ");
        cleaner.add_item("Banana");
        cleaner.add_item("cherry");

        cleaner.clean();

        let results = cleaner.get_results();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "apple");
        assert_eq!(results[1], "banana");
        assert_eq!(results[2], "cherry");
    }
}