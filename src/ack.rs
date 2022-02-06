use std::str::FromStr;

use super::nuclide::Nuclide;
use super::ParseError;

pub struct Entry {
    pub nuclide: Nuclide,
    // half_life: HalfLife,
    pub records: u64,
}

impl FromStr for Entry {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            nuclide: s[0..7].parse()?,
            records: s[24..32]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidInteger(s[24..32].trim().to_string()))?,
        })
    }
}

#[derive(Debug)]
pub struct Spectrum {
    r#yield: f64,
    energy: f64,
    transition: Vec<String>,
}

impl FromStr for Spectrum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r#yield = s[0..11]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidFloat(s[0..11].trim().to_string()))?;
        let energy = s[11..23]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidFloat(s[11..23].trim().to_string()))?;
        let mut transition = vec![];
        for s in s[23..32].trim().split_whitespace() {
            transition.push(s.to_string());
        }
        Ok(Self {
            r#yield,
            energy,
            transition,
        })
    }
}
