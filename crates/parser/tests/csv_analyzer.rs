
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub numeric_columns: Vec<String>,
    pub text_columns: Vec<String>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut numeric_flags = vec![true; column_count];
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        for (i, field) in record.iter().enumerate() {
            if i < column_count && numeric_flags[i] {
                if field.parse::<f64>().is_err() && !field.is_empty() {
                    numeric_flags[i] = false;
                }
            }
        }
    }
    
    let mut numeric_columns = Vec::new();
    let mut text_columns = Vec::new();
    
    for (i, header) in headers.iter().enumerate() {
        if i < numeric_flags.len() && numeric_flags[i] {
            numeric_columns.push(header.to_string());
        } else {
            text_columns.push(header.to_string());
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count,
        numeric_columns,
        text_columns,
    })
}

pub fn filter_csv<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    predicate: impl Fn(&csv::StringRecord) -> bool,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    
    let mut rdr = csv::Reader::from_reader(input_file);
    let mut wtr = csv::Writer::from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;
    
    let mut filtered_count = 0;
    
    for result in rdr.records() {
        let record = result?;
        if predicate(&record) {
            wtr.write_record(&record)?;
            filtered_count += 1;
        }
    }
    
    wtr.flush()?;
    Ok(filtered_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,age,salary\")?;
        writeln!(temp_file, "1,Alice,30,50000\")?;
        writeln!(temp_file, "2,Bob,25,45000\")?;
        writeln!(temp_file, "3,Charlie,35,60000\")?;
        
        let stats = analyze_csv(temp_file.path())?;
        
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns, vec!["id", "age", "salary"]);
        assert_eq!(stats.text_columns, vec!["name"]);
        
        Ok(())
    }
    
    #[test]
    fn test_filter_csv() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "id,name,age\")?;
        writeln!(input_file, "1,Alice,30\")?;
        writeln!(input_file, "2,Bob,25\")?;
        writeln!(input_file, "3,Charlie,35\")?;
        
        let output_file = NamedTempFile::new()?;
        
        let filtered = filter_csv(
            input_file.path(),
            output_file.path(),
            |record| {
                record.get(2)
                    .and_then(|age| age.parse::<i32>().ok())
                    .map(|age| age >= 30)
                    .unwrap_or(false)
            },
        )?;
        
        assert_eq!(filtered, 2);
        
        let mut rdr = csv::Reader::from_path(output_file.path())?;
        let records: Vec<_> = rdr.records().collect::<Result<_, _>>()?;
        assert_eq!(records.len(), 2);
        
        Ok(())
    }
}