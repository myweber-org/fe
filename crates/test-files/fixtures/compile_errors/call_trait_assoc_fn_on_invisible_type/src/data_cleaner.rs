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
    let mut csv_writer = WriterBuilder::new().from_writer(writer);
    
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
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv_removes_empty_rows() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,,London").unwrap();
        writeln!(input_file, ",25,Berlin").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output_content, "name,age,city\nAlice,30,New York\n");
    }
}