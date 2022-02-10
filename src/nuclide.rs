use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serde_plain::derive_deserialize_from_fromstr;
use std::hash::Hash;
use uom::si::f64::Time;
use uom::si::time::{day, hour, microsecond, millisecond, minute, second, year};

use super::ParseError;
use crate::regex;

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Clone)]
enum MetastableState {
    #[serde(rename = "m")]
    M,
    #[serde(rename = "n")]
    N,
    #[serde(rename = "")]
    None,
}

serde_plain::derive_fromstr_from_deserialize!(MetastableState);
serde_plain::derive_display_from_serialize!(MetastableState);

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Nuclide {
    symbol: String,
    mass_number: u64,
    meta: MetastableState,
}

impl std::fmt::Debug for Nuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.symbol == "SF" {
            // spontaneous fission
            write!(f, "{}", self.symbol)
        } else {
            write!(f, "{}-{}{}", self.symbol, self.mass_number, self.meta)
        }
    }
}

impl std::str::FromStr for Nuclide {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<symbol>\w+)-(?P<mass>\d+)(?P<meta>[mn])?");

        if s == "SF" {
            Ok(Self {
                symbol: "SF".to_string(),
                mass_number: 0,
                meta: MetastableState::None,
            })
        } else {
            let captures = re
                .captures(s)
                .ok_or_else(|| ParseError::InvalidNuclide(s.to_string()))?;

            let symbol = captures.name("symbol").unwrap().as_str().to_string();
            let mass_number = captures.name("mass").unwrap().as_str().parse().unwrap();
            let meta = captures
                .name("meta")
                .map_or("", |c| c.as_str())
                .parse()
                .unwrap();

            Ok(Self {
                symbol,
                mass_number,
                meta,
            })
        }
    }
}

serde_plain::derive_deserialize_from_fromstr!(Nuclide, "invalid nuclide");

#[derive(Copy, Clone)]
pub struct HalfLife(uom::si::f64::Time);

impl std::fmt::Debug for HalfLife {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::str::FromStr for HalfLife {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<value>\d+\.?(?:\d+)?(?:[Ee][+-]?\d+)?)(?P<unit>(?:[um]?s)|m|h|d|y)");

        // captured values should be valid
        match re.captures(s) {
            Some(captures) => match captures.name("value").unwrap().as_str().parse::<f64>() {
                Ok(value) => {
                    let t = match captures.name("unit").unwrap().as_str() {
                        "us" => Ok(Time::new::<microsecond>(value)),
                        "ms" => Ok(Time::new::<millisecond>(value)),
                        "s" => Ok(Time::new::<second>(value)),
                        "m" => Ok(Time::new::<minute>(value)),
                        "h" => Ok(Time::new::<hour>(value)),
                        "d" => Ok(Time::new::<day>(value)),
                        "y" => Ok(Time::new::<year>(value)),
                        _ => Err(ParseError::InvalidHalfLife(s.to_string())),
                    }?;
                    Ok(HalfLife(t))
                }
                Err(_) => Err(ParseError::InvalidHalfLife(s.to_string())),
            },
            None => Err(ParseError::InvalidHalfLife(s.to_string())),
        }
    }
}

derive_deserialize_from_fromstr!(HalfLife, "invalid half-life");

bitflags! {
    pub struct DecayMode: u8 {
        const ALPHA = 0x01;
        const BETA_MINUS = 0x02;
        const BETA_PLUS_OR_EC = 0x04;
        const ELECTRON_CAPTURE = 0x08;
        const ISOMETRIC_TRANSITION = 0x10;
        const SPONTANEOUS_FISSION = 0x20;
    }
}

impl std::str::FromStr for DecayMode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"A|B\-|ECB\+|EC|IT|SF");

        let mut dm: DecayMode = DecayMode::empty();
        for captures in re.captures_iter(s) {
            for capture in captures.iter() {
                match capture.unwrap().as_str() {
                    "A" => dm |= DecayMode::ALPHA,
                    "B-" => dm |= DecayMode::BETA_MINUS,
                    "ECB+" => dm |= DecayMode::BETA_PLUS_OR_EC,
                    "EC" => dm |= DecayMode::ELECTRON_CAPTURE,
                    "IT" => dm |= DecayMode::ISOMETRIC_TRANSITION,
                    "SF" => dm |= DecayMode::SPONTANEOUS_FISSION,
                    dm => return Err(ParseError::InvalidDecayMode(dm.to_string())),
                }
            }
        }

        Ok(dm)
    }
}

derive_deserialize_from_fromstr!(DecayMode, "invalid decay mode");
