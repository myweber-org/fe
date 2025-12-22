
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvFilter {
    input_path: String,
    output_path: String,
    selected_columns: Vec<usize>,
    delimiter: char,
}

impl CsvFilter {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvFilter {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            selected_columns: Vec::new(),
            delimiter: ',',
        }
    }

    pub fn select_columns(&mut self, columns: &[usize]) -> &mut Self {
        self.selected_columns = columns.to_vec();
        self
    }

    pub fn set_delimiter(&mut self, delimiter: char) -> &mut Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(self.delimiter).collect();

            if line_num == 0 {
                self.write_header(&parts, &mut output_file)?;
            } else {
                self.write_row(&parts, &mut output_file)?;
            }
        }

        Ok(())
    }

    fn write_header(&self, headers: &[&str], output: &mut File) -> io::Result<()> {
        let selected_headers: Vec<&str> = if self.selected_columns.is_empty() {
            headers.to_vec()
        } else {
            self.selected_columns
                .iter()
                .filter_map(|&idx| headers.get(idx))
                .copied()
                .collect()
        };

        writeln!(output, "{}", selected_headers.join(&self.delimiter.to_string()))
    }

    fn write_row(&self, row: &[&str], output: &mut File) -> io::Result<()> {
        let selected_cells: Vec<&str> = if self.selected_columns.is_empty() {
            row.to_vec()
        } else {
            self.selected_columns
                .iter()
                .filter_map(|&idx| row.get(idx))
                .copied()
                .collect()
        };

        writeln!(output, "{}", selected_cells.join(&self.delimiter.to_string()))
    }
}

pub fn filter_csv(
    input: &str,
    output: &str,
    columns: Option<&[usize]>,
    delimiter: Option<char>,
) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvFilter::new(input, output);

    if let Some(cols) = columns {
        processor.select_columns(cols);
    }

    if let Some(delim) = delimiter {
        processor.set_delimiter(delim);
    }

    processor.process()
}