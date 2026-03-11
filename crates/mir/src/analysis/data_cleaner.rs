use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let mut seen = HashSet::new();
    for result in csv_reader.records() {
        let record = result?;
        let row_key: String = record.iter().collect();
        
        if seen.insert(row_key) {
            csv_writer.write_record(&record)?;
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
    fn test_remove_duplicates() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,name,value").unwrap();
        writeln!(input_file, "1,Alice,100").unwrap();
        writeln!(input_file, "2,Bob,200").unwrap();
        writeln!(input_file, "1,Alice,100").unwrap();
        writeln!(input_file, "3,Charlie,300").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        remove_duplicates(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let expected_lines = vec![
            "id,name,value",
            "1,Alice,100",
            "2,Bob,200",
            "3,Charlie,300",
        ];
        
        for (i, line) in output_content.lines().enumerate() {
            assert_eq!(line, expected_lines[i]);
        }
    }
}