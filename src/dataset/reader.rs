use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::Error;

pub struct FileReader(BufReader<File>);

impl FileReader {
    pub fn new(path: &Path) -> Self {
        Self(BufReader::new(File::open(path).unwrap()))
    }

    pub fn skip_lines(mut self, n: usize) -> Self {
        let mut buf = vec![];
        for _ in 0..n {
            self.0.read_until(b'\n', &mut buf).unwrap();
        }
        self
    }

    pub fn read_line(&mut self, buf: &mut String) -> Result<usize, Error> {
        buf.clear();
        self.0.read_line(buf).map_err(std::convert::Into::into)
    }
}
