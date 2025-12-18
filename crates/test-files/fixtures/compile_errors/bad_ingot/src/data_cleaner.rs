use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();
        
        if filtered_record.len() == headers.len() {
            csv_writer.write_record(&filtered_record)?;
        }
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv() {
        let input_data = "name,age,city\nJohn,25,NYC\nJane,,London\nBob,30,\n,40,Boston";
        let input_temp = NamedTempFile::new().unwrap();
        fs::write(input_temp.path(), input_data).unwrap();
        
        let output_temp = NamedTempFile::new().unwrap();
        
        clean_csv(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap()
        ).unwrap();
        
        let output = fs::read_to_string(output_temp.path()).unwrap();
        assert_eq!(output, "name,age,city\nJohn,25,NYC\n");
    }
}