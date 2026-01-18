
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
struct CsvRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl CsvRecord {
    fn from_row(row: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        Ok(CsvRecord {
            id: row[0].parse()?,
            name: row[1].to_string(),
            value: row[2].parse()?,
            category: row[3].to_string(),
        })
    }
}

struct CsvAnalyzer {
    records: Vec<CsvRecord>,
}

impl CsvAnalyzer {
    fn new() -> Self {
        CsvAnalyzer {
            records: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        
        for result in reader.records() {
            let record = result?;
            self.records.push(CsvRecord::from_row(&record)?);
        }
        
        Ok(())
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn generate_summary(&self) -> String {
        let avg = self.calculate_average();
        let max_record = self.find_max_value();
        let categories: std::collections::HashSet<_> = 
            self.records.iter().map(|r| &r.category).collect();
        
        format!(
            "Records: {}, Average: {:.2}, Categories: {}, Max Value: {}",
            self.records.len(),
            avg,
            categories.len(),
            max_record.map_or("None".to_string(), |r| r.value.to_string())
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut analyzer = CsvAnalyzer::new();
    
    match analyzer.load_from_file("data.csv") {
        Ok(_) => {
            println!("{}", analyzer.generate_summary());
            
            let filtered = analyzer.filter_by_category("premium");
            println!("Premium records: {}", filtered.len());
            
            if let Some(max) = analyzer.find_max_value() {
                println!("Highest value: {} ({})", max.value, max.name);
            }
        }
        Err(e) => eprintln!("Error loading CSV: {}", e),
    }
    
    Ok(())
}