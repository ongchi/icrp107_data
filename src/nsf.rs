use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use super::{nuclide::Nuclide, ParseError};

pub struct Entry {
    pub nuclide: Nuclide,
    // br: f64,
    pub records: u64,
}

impl FromStr for Entry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            nuclide: s[0..7].parse()?,
            records: s[20..29]
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .parse()
                .map_err(|_| ParseError::InvalidInteger(s[20..29].trim().to_string()))?,
        })
    }
}

#[derive(Debug)]
pub struct Spectrum {
    energy_lower: f64,
    energy_upper: f64,
    r#yield: f64,
}

impl FromStr for Spectrum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            energy_lower: s[0..8]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidFloat(s[0..8].trim().to_string()))?,
            energy_upper: s[8..17]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidFloat(s[8..17].trim().to_string()))?,
            r#yield: s[17..29]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidFloat(s[17..29].trim().to_string()))?,
        })
    }
}

#[derive(Debug)]
pub struct NsfData(HashMap<Nuclide, Vec<Spectrum>>);

impl NsfData {
    pub fn new<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let mut nsf = BufReader::new(File::open(path.as_ref().join("ICRP-07.NSF")).unwrap());
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while nsf
            .read_line(&mut buf)
            .map_err(|e| ParseError::UnexpectedError(e.into()))?
            != 0
        {
            let nuc_record: Entry = buf.parse()?;
            println!("{:?}", &nuc_record.nuclide);

            let mut nsf_data = vec![];
            for _ in 0..(nuc_record.records) {
                buf.clear();
                nsf.read_line(&mut buf)
                    .map_err(|e| ParseError::UnexpectedError(e.into()))?;
                nsf_data.push(buf.parse()?);
            }
            inner.insert(nuc_record.nuclide, nsf_data);

            buf.clear();
        }

        Ok(Self(inner))
    }
}
