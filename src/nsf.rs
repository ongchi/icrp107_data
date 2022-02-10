use std::str::FromStr;

use crate::FromRow;

use super::ParseError;

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
            energy_lower: s.from_row(0..8)?,
            energy_upper: s.from_row(8..17)?,
            r#yield: s.from_row(17..29)?,
        })
    }
}
