use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::str::FromStr;

use super::{RadiationType, Spectrum};
use crate::derive_from_str_from_fixed_width;
use crate::error::Error;

#[derive(Debug, FixedWidth, Deserialize)]
pub struct RadSpectrum {
    #[fixed_width(range = "26..29")]
    r#type: RadiationType,

    // yield (/nt)
    #[fixed_width(range = "2..14")]
    r#yield: f64,

    // energy of reaidation (MeV)
    #[fixed_width(range = "14..26")]
    energy: f64,
}

derive_from_str_from_fixed_width!(RadSpectrum);

impl From<RadSpectrum> for Spectrum {
    fn from(rad: RadSpectrum) -> Self {
        Self::Radiation {
            r#type: rad.r#type,
            r#yield: rad.r#yield,
            energy: rad.energy,
        }
    }
}
