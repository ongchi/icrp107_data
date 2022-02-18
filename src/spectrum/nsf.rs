use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::str::FromStr;

use super::Spectrum;
use crate::derive_from_str_from_fixed_width;
use crate::error::Error;

#[derive(Debug, FixedWidth, Deserialize)]
pub struct NsfSpectrum {
    #[fixed_width(range = "0..8")]
    energy_lower: f64,
    #[fixed_width(range = "8..17")]
    energy_upper: f64,
    #[fixed_width(range = "17..29")]
    r#yield: f64,
}

derive_from_str_from_fixed_width!(NsfSpectrum);

impl From<NsfSpectrum> for Spectrum {
    fn from(nsf: NsfSpectrum) -> Self {
        Self::SpontaneousFissionNeutron {
            energy_lower: nsf.energy_lower,
            energy_upper: nsf.energy_upper,
            r#yield: nsf.r#yield,
        }
    }
}
