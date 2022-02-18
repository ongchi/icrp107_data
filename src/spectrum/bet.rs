use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::str::FromStr;

use super::Spectrum;
use crate::derive_from_str_from_fixed_width;
use crate::error::Error;

// beta- emission data
#[derive(Debug, FixedWidth, Deserialize)]
pub struct BetSpectrum {
    // energy grid point (MeV)
    #[fixed_width(range = "0..7")]
    energy: f64,

    // number of beta particles per MeV per nuclear transformation
    #[fixed_width(range = "7..17")]
    number: f64,
}

derive_from_str_from_fixed_width!(BetSpectrum);

impl From<BetSpectrum> for Spectrum {
    fn from(bet: BetSpectrum) -> Self {
        Self::Beta {
            energy: bet.energy,
            number: bet.number,
        }
    }
}
