use fixed_width::Field;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::error::Error;
use crate::ndx::{Attribute, NdxEntry};
use crate::regex;
use crate::Nuclide;

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
        self.0
            .read_line(buf)
            .map_err(|e| Error::Unexpected(e.into()))
    }
}

pub struct IndexReader {
    reader: FileReader,
}

impl IndexReader {
    pub fn new(path: &Path) -> Self {
        Self {
            reader: FileReader::new(path).skip_lines(1),
        }
    }

    pub fn read(&mut self) -> Result<HashMap<Nuclide, Attribute>, Error> {
        let mut ndx = HashMap::new();

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let row: NdxEntry =
                fixed_width::from_str(&buf).map_err(|e| Error::Unexpected(e.into()))?;
            ndx.insert(row.nuclide, row.into());
        }

        Ok(ndx)
    }
}

pub struct SpectrumReader<T> {
    reader: FileReader,
    _marker: std::marker::PhantomData<T>,
}

impl<T> SpectrumReader<T>
where
    T: FromStr<Err = Error>,
{
    pub fn new(path: &Path) -> Self {
        Self {
            reader: FileReader::new(path),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn read(&mut self) -> Result<HashMap<Nuclide, Vec<T>>, Error> {
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let nuclide: Nuclide = (&buf[0..7]).parse()?;
            let records = &buf[7..].replace("\0", " ");
            let records = records
                .split_whitespace()
                .last()
                .ok_or_else(|| {
                    Error::Unexpected(anyhow::anyhow!("failed to get spectrum for {}", nuclide))
                })?
                .parse()
                .map_err(|_| Error::InvalidInteger(records.trim().to_string()))?;

            let mut spectrum = vec![];
            for _ in 0..(records) {
                self.reader.read_line(&mut buf)?;
                spectrum.push(buf.parse()?);
            }
            inner.insert(nuclide, spectrum);
        }

        Ok(inner)
    }
}

pub(crate) fn fields_from_fortran_format(fmt: &str) -> Result<(usize, Vec<Field>), String> {
    let re = regex!(
        r"(?P<repeat>\d*)(?:(?P<type>[a-z]{1,2})|(?P<nested>\([^\(\)]+?\)))(?:(?P<length>\d+)(?:\.\d+)?)?"
    );
    let mut start = 0;
    let mut fields = vec![];

    let fmt: String = fmt
        .trim_matches(|c: char| c.is_whitespace() || c == '(' || c == ')')
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let captures: Vec<_> = re.captures_iter(&fmt).collect();
    if captures.is_empty() {
        Err("invalid fortran format".to_string())
    } else {
        for cap in captures.into_iter() {
            let r: usize = cap
                .name("repeat")
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap();

            match cap.name("type") {
                Some(m) => {
                    let t = m.as_str();
                    let l: usize = cap
                        .name("length")
                        .map(|m| m.as_str().parse().unwrap())
                        .unwrap_or(1);
                    match t {
                        "x" => start += r * l,
                        _ => {
                            for _ in 0..r {
                                fields.push(Field::default().range(start..start + l));
                                start += l
                            }
                        }
                    }
                }
                None => match cap.name("nested") {
                    Some(m) => {
                        if cap.name("length").is_some() {
                            return Err("invalid fortran format".to_string());
                        } else {
                            let fmt = m.as_str();
                            let (l, nested_fields) = fields_from_fortran_format(fmt)?;
                            for _ in 0..r {
                                for field in nested_fields.iter() {
                                    let mut field = field.clone();
                                    field.range.start += start;
                                    field.range.end += start;
                                    fields.push(field)
                                }
                                start += l;
                            }
                        }
                    }
                    None => return Err("invalid fortran format".to_string()),
                },
            }
        }

        Ok((start, fields))
    }
}
