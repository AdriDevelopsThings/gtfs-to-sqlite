use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, BufReader, Read},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};

const BUF_SIZE: usize = 64 * 1024;

pub struct CsvReader<R>
where
    R: Read,
{
    buf: BufReader<R>,
    header_len: usize,
    headers: Vec<String>,
    header: Arc<HashMap<String, usize>>,
    size: usize,
    already_processed: usize,
}

pub struct Row {
    pub row: Vec<String>,
    header: Arc<HashMap<String, usize>>,
}

impl<R> CsvReader<R>
where
    R: Read,
{
    pub fn new(reader: R, size: usize) -> Result<Self> {
        let mut s = Self {
            buf: BufReader::with_capacity(BUF_SIZE, reader),
            header_len: 0,
            headers: Vec::new(),
            header: Arc::new(HashMap::new()),
            size,
            already_processed: 0,
        };

        s.read_header()?;

        Ok(s)
    }

    fn read_line(&mut self) -> Result<Option<String>> {
        let mut buf = String::new();
        let n = self.buf.read_line(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        buf.pop();
        self.already_processed += n;
        Ok(Some(buf))
    }

    fn read_line_cols(&mut self, col_size_guess: Option<usize>) -> Result<Option<Vec<String>>> {
        let line = match self.read_line()? {
            Some(l) => l,
            None => return Ok(None),
        };
        let line = line.as_bytes();
        // let chars = line.chars().collect::<Vec<char>>();

        let mut cols: Vec<String> = if let Some(col_size_guess) = col_size_guess {
            Vec::with_capacity(col_size_guess)
        } else {
            Vec::new()
        };
        let mut quotes = false;
        let mut quotes_count: usize = 0;
        let mut word_start: usize = 0;

        //for (i, c) in chars.iter().enumerate() {
        for i in 0..line.len() {
            let c = line[i];
            if c == b'"' {
                if word_start == i {
                    word_start += 1;
                }
                quotes_count += 1;
                quotes = !quotes;
            } else if !quotes && c == b',' {
                let word_end = if i > word_start && line[i - 1] == b'"' {
                    i - 1
                } else {
                    i
                };
                let field = &line[word_start..word_end];
                let field = unsafe { String::from_utf8_unchecked(field.to_vec()) };
                if quotes_count > 2 {
                    cols.push(field.replace("\"\"", "\""));
                } else {
                    cols.push(field);
                }
                word_start = i + 1;
                quotes_count = 0;
            }
        }
        let word_end = if line.len() > word_start && line[line.len() - 1] == b'"' {
            line.len() - 1
        } else {
            line.len()
        };
        let field = &line[word_start..word_end];
        let field = unsafe { String::from_utf8_unchecked(field.to_vec()) };
        cols.push(field.replace("\"\"", "\""));

        Ok(Some(cols))
    }

    fn read_header(&mut self) -> Result<()> {
        let cols = self
            .read_line_cols(None)?
            .context("CSV file seems to be malformed because first line is empty")?;
        self.headers = cols.clone();
        self.header_len = cols.len();
        let mut header = HashMap::with_capacity(self.header_len);
        for (i, c) in cols.into_iter().enumerate() {
            header.insert(c, i);
        }

        self.header = Arc::new(header);

        Ok(())
    }

    pub fn read_row(&mut self) -> Result<Option<Row>> {
        let cols = match self.read_line_cols(Some(self.header_len))? {
            Some(c) => c,
            None => return Ok(None),
        };

        if cols.len() != self.header_len {
            return Err(anyhow!(
                "Invalid csv row, headers are {:?} and row is {:?}",
                self.header.keys(),
                cols
            ));
        }

        Ok(Some(Row {
            row: cols,
            header: self.header.clone(),
        }))
    }

    pub fn processed(&self) -> f32 {
        (self.already_processed as f32) / (self.size as f32)
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Row {{ ")?;
        for (key, i) in self.header.iter() {
            write!(f, "\"{key}\" = \"{}\" ", self.row[*i])?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
