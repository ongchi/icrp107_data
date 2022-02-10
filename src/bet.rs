use std::str::FromStr;

use crate::FromRow;

use super::ParseError;

#[derive(Debug)]
pub struct Spectrum {
    // energy grid point (MeV)
    energy: f64,
    // number of beta particles per MeV per nuclear transformation
    number: f64,
}

impl FromStr for Spectrum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            energy: s.from_row(0..7)?,
            number: s.from_row(7..17)?,
        })
    }
}
