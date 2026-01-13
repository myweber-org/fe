use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            csv_writer.serialize(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered.csv";
    let threshold = 50.0;

    process_csv(input_file, output_file, threshold)?;
    println!("Filtered data saved to {}", output_file);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,active\n1,test1,30.5,true\n2,test2,75.2,true\n3,test3,45.0,false\n";
        
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "{}", input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_csv(input_file.path().to_str().unwrap(), 
                   output_file.path().to_str().unwrap(), 
                   50.0)?;

        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("test2"));
        assert!(!output_content.contains("test1"));
        assert!(!output_content.contains("test3"));
        
        Ok(())
    }
}