use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files<P: AsRef<Path>>(
    input_paths: &[P],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, path) in input_paths.iter().enumerate() {
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if index == 0 {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        } else if headers != rdr.headers()? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CSV headers do not match",
            )
            .into());
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() -> Result<(), Box<dyn Error>> {
        let csv1_content = "name,age\nAlice,30\nBob,25";
        let csv2_content = "name,age\nCharlie,35\nDiana,28";

        let file1 = NamedTempFile::new()?;
        let file2 = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        std::fs::write(&file1, csv1_content)?;
        std::fs::write(&file2, csv2_content)?;

        let inputs = [file1.path(), file2.path()];
        merge_csv_files(&inputs, output_file.path())?;

        let mut merged_content = String::new();
        std::fs::File::open(output_file.path())?.read_to_string(&mut merged_content)?;

        let expected = "name,age\nAlice,30\nBob,25\nCharlie,35\nDiana,28\n";
        assert_eq!(merged_content, expected);
        Ok(())
    }
}