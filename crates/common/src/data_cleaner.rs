use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                field
                    .trim()
                    .to_lowercase()
                    .replace("\"", "")
                    .replace("'", "")
            })
            .collect();
        csv_writer.write_record(&cleaned_record)?;
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
        let input_data = "name,age,city\n\"John\",25,\"New York\"\nJane ,30,'London'\n";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        clean_csv(input_file.path().to_str().unwrap(), output_path).unwrap();

        let output_content = fs::read_to_string(output_path).unwrap();
        let expected = "name,age,city\njohn,25,new york\njane,30,london\n";
        assert_eq!(output_content, expected);
    }
}