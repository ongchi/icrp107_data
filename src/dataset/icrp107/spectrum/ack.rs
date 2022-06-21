use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::str::FromStr;

use super::Spectrum;
use crate::derive_from_str;
use crate::error::Error;

#[derive(Debug, FixedWidth, Deserialize)]
pub struct AckSpectrum {
    // yield (/nt)
    #[fixed_width(range = "0..11")]
    r#yield: f64,

    // energy of reaidation (eV)
    #[fixed_width(range = "11..23")]
    energy: f64,

    #[fixed_width(range = "23..32")]
    transition: String,
}

derive_from_str!(AckSpectrum);

impl From<AckSpectrum> for Spectrum {
    fn from(ack: AckSpectrum) -> Self {
        Self::AugerCosterKronigElectron {
            r#yield: ack.r#yield,
            energy: ack.energy,
            transition: ack.transition,
        }
    }
}
