mod ack;
mod bet;
mod ndx;
mod nsf;
mod nuclide;
mod rad;
mod spectrum;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Range;
use std::path::Path;

pub use ndx::NuclideData;
pub use nuclide::Nuclide;
use rad::RadiationType;

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

struct FileReader(BufReader<File>);

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

    pub fn read_buf(&mut self, buf: &mut String) -> Result<usize, ParseError> {
        buf.clear();
        self.0
            .read_line(buf)
            .map_err(|e| ParseError::UnexpectedError(e.into()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("invalid nuclide: {0}")]
    InvalidNuclide(String),
    #[error("invalid half life: {0}")]
    InvalidHalfLife(String),
    #[error("invalid decay mode: {0}")]
    InvalidDecayMode(String),
    #[error("invalid radiation type: {0}")]
    InvalidRadiationType(String),
    #[error("invalid float number: {0}")]
    InvalidFloat(String),
    #[error("invalid integer: {0}")]
    InvalidInteger(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub trait FromRow<T: Sized> {
    fn from_row(&self, range: Range<usize>) -> Result<T, ParseError>;
}

impl<T> FromRow<Option<T>> for str
where
    str: FromRow<T>,
{
    fn from_row(&self, range: Range<usize>) -> Result<Option<T>, ParseError> {
        Ok(
            match self
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .len()
            {
                0 => None,
                _ => Some(self.from_row(range)?),
            },
        )
    }
}

macro_rules! derive_fromrow {
    ($type:ty, $err:ident) => {
        impl FromRow<$type> for str {
            fn from_row(&self, range: Range<usize>) -> Result<$type, ParseError> {
                let s = &self[range];
                s.trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .parse()
                    .map_err(|_| ParseError::$err(s.to_string()))
            }
        }
    };
}

derive_fromrow!(u64, InvalidInteger);
derive_fromrow!(f64, InvalidFloat);
derive_fromrow!(RadiationType, InvalidRadiationType);

impl FromRow<Vec<String>> for str {
    fn from_row(&self, range: Range<usize>) -> Result<Vec<String>, ParseError> {
        Ok(self[range]
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect())
    }
}
