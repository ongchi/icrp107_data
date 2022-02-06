use std::str::FromStr;

use super::nuclide::Nuclide;
use super::ParseError;

pub struct Entry {
    pub nuclide: Nuclide,
    pub records: u64,
}

impl FromStr for Entry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            nuclide: s[0..7].parse()?,
            records: s[7..17]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidInteger(s[7..17].trim().to_string()))?,
        })
    }
}

serde_plain::derive_deserialize_from_fromstr!(Entry, "invalid beta entry");

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
        let energy = s[0..7]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidFloat(s[0..7].trim().to_string()))?;
        let number = s[7..17]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidFloat(s[7..17].trim().to_string()))?;
        Ok(Self { energy, number })
    }
}

serde_plain::derive_deserialize_from_fromstr!(Spectrum, "invalid emitted beta");
