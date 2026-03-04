use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
struct CsvStats {
    row_count: usize,
    column_count: usize,
    has_header: bool,
    sample_data: Vec<Vec<String>>,
}

impl CsvStats {
    fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            has_header: false,
            sample_data: Vec::new(),
        }
    }

    fn analyze<P: AsRef<Path>>(path: P, sample_size: usize) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers = rdr.headers()?.clone();
        let mut stats = CsvStats::new();
        
        stats.has_header = !headers.is_empty();
        stats.column_count = headers.len();
        
        let mut records = rdr.records();
        let mut sample_collected = 0;
        
        while let Some(result) = records.next() {
            let record = result?;
            stats.row_count += 1;
            
            if sample_collected < sample_size {
                let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                stats.sample_data.push(row_data);
                sample_collected += 1;
            }
        }
        
        Ok(stats)
    }
    
    fn display(&self) {
        println!("CSV Analysis Results:");
        println!("Total Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Has Header: {}", self.has_header);
        
        if !self.sample_data.is_empty() {
            println!("\nSample Data (first {} rows):", self.sample_data.len());
            for (i, row) in self.sample_data.iter().enumerate() {
                println!("Row {}: {:?}", i + 1, row);
            }
        }
    }
}

fn validate_csv_structure<P: AsRef<Path>>(path: P) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?;
    let expected_columns = headers.len();
    
    for result in rdr.records() {
        let record = result?;
        if record.len() != expected_columns {
            return Ok(false);
        }
    }
    
    Ok(true)
}

fn main() -> Result<(), Box<dyn Error>> {
    let test_file = "data/sample.csv";
    
    match CsvStats::analyze(test_file, 3) {
        Ok(stats) => {
            stats.display();
            
            let is_valid = validate_csv_structure(test_file)?;
            println!("\nCSV Structure Valid: {}", is_valid);
        }
        Err(e) => eprintln!("Error analyzing CSV: {}", e),
    }
    
    Ok(())
}