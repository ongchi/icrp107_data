use std::str::FromStr;

use crate::FromRow;

use super::ParseError;

#[derive(Debug)]
pub struct Spectrum {
    r#yield: f64,
    energy: f64,
    transition: Vec<String>,
}

impl FromStr for Spectrum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            r#yield: s.from_row(0..11)?,
            energy: s.from_row(11..23)?,
            transition: s.from_row(23..32)?,
        })
    }
}
