use serde::{Deserialize, Serialize};
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
            nuclide: s[0..7].parse::<Nuclide>()?,
            records: s[20..29]
                .trim()
                .parse::<u64>()
                .map_err(|_| ParseError::InvalidInteger(s[20..29].trim().to_string()))?,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum RadiationType {
    G,
    PG,
    DG,
    X,
    AQ,
    #[serde(rename = "B+")]
    BPlus,
    #[serde(rename = "B-")]
    BMinus,
    BD,
    IE,
    AE,
    A,
    AR,
    FF,
    N,
    // ??
    DB,
}

serde_plain::derive_fromstr_from_deserialize!(RadiationType);

impl RadiationType {
    fn code(&self) -> u8 {
        match self {
            Self::G => 1,
            Self::PG => 1,
            Self::DG => 1,
            Self::X => 2,
            Self::AQ => 3,
            Self::BPlus => 4,
            Self::BMinus => 5,
            Self::BD => 5,
            Self::IE => 6,
            Self::AE => 7,
            Self::A => 8,
            Self::AR => 9,
            Self::FF => 10,
            Self::N => 11,
            Self::DB => 0,
        }
    }
}

#[derive(Debug)]
pub struct Spectrum {
    r#type: RadiationType,
    // yield of radiation (/nt)
    r#yield: f64,
    // energy of reaidation (MeV)
    energy: f64,
}

impl FromStr for Spectrum {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            r#type: s[26..29]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidRadiationType(s[26..29].trim().to_string()))?,
            r#yield: s[2..14]
                .trim()
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidFloat(s[2..14].trim().to_string()))?,
            energy: s[14..26]
                .trim()
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidFloat(s[14..26].trim().to_string()))?,
        })
    }
}

serde_plain::derive_deserialize_from_fromstr!(Spectrum, "invalid emitted radiation");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radiation_type() {
        assert_eq!("G".parse::<RadiationType>().unwrap(), RadiationType::G);
        assert_eq!("B+".parse::<RadiationType>().unwrap(), RadiationType::BPlus);
        assert_eq!(
            "B-".parse::<RadiationType>().unwrap(),
            RadiationType::BMinus
        );
    }
}
