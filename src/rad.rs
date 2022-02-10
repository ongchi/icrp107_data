use serde::Deserialize;
use std::str::FromStr;

use crate::FromRow;

use super::ParseError;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum RadiationType {
    #[serde(rename = "G")]
    Gamma,
    #[serde(rename = "PG")]
    PromptGamma,
    #[serde(rename = "DG")]
    DelayedGamma,
    X,
    #[serde(rename = "AQ")]
    AnnihilationPhoton,
    #[serde(rename = "B+")]
    BetaPlus,
    #[serde(rename = "B-")]
    BetaMinus,
    #[serde(rename = "DB")]
    DelayedBeta,
    #[serde(rename = "IE")]
    InternalConversionElectron,
    #[serde(rename = "AE")]
    AugerElectron,
    #[serde(rename = "A")]
    Alpha,
    #[serde(rename = "AR")]
    AlphaRecoil,
    #[serde(rename = "FF")]
    FissionFragment,
    #[serde(rename = "N")]
    Neutron,
}

serde_plain::derive_fromstr_from_deserialize!(RadiationType);

#[derive(Debug)]
pub struct Spectrum {
    code: u64,

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
            code: s.from_row(0..2)?,
            r#type: s.from_row(26..29)?,
            r#yield: s.from_row(2..14)?,
            energy: s.from_row(14..26)?,
        })
    }
}
