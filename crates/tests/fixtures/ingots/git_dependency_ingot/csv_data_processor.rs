
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_and_validate<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            
            if self.validate_record(&fields) {
                records.push(fields);
            } else {
                return Err("Invalid record format".into());
            }
        }
        
        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.trim().is_empty())
    }

    pub fn transform_data(&self, data: Vec<Vec<String>>) -> Vec<Vec<String>> {
        data.into_iter()
            .map(|record| {
                record.into_iter()
                    .map(|field| field.to_uppercase())
                    .collect()
            })
            .collect()
    }

    pub fn write_transformed_data<P: AsRef<Path>>(
        &self, 
        data: Vec<Vec<String>>, 
        output_path: P
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(self.delimiter as u8)
            .from_writer(file);

        for record in data {
            wtr.write_record(&record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city\nJohn,25,New York\nJane,30,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let data = processor.read_and_validate(temp_file.path()).unwrap();
        assert_eq!(data.len(), 2);
        
        let transformed = processor.transform_data(data);
        assert_eq!(transformed[0][0], "JOHN");
        
        let output_file = NamedTempFile::new().unwrap();
        processor.write_transformed_data(transformed, output_file.path()).unwrap();
    }
}