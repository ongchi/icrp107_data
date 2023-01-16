use crate::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Pathway {
    AirSubmersion,
    WaterImmersion,
    GroundSurface,
    SoilOneCm,
    SoilFiveCm,
    SoilFifteenCm,
    SoilInfinite,
    Ingestion,
    Inhalation,
}

serde_plain::derive_display_from_serialize!(Pathway);
serde_plain::derive_fromstr_from_deserialize!(Pathway, |e| -> Error {
    Error::InvalidPathway(e.to_string())
});

#[derive(Debug, PartialEq)]
pub struct BiokineticAttr {
    /// Chemical compound
    pub compound: String,
    /// Gastroinstestinal absorption factor
    pub f1: f64,
    /// Clearance class or pulmonary absorption type
    pub respiratory_tract_attr: Option<RespiratoryTractAttr>,
}

#[derive(Debug, PartialEq)]
pub enum RespiratoryTractAttr {
    ICRP30(ClearanceClass),
    ICRP66(PulmonaryAbsorptionType),
}

/// Pulmonary Absorption Type of respiratory tract model of ICRP 66
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PulmonaryAbsorptionType {
    #[serde(rename = "F")]
    Fast,
    #[serde(rename = "M")]
    Moderate,
    #[serde(rename = "S")]
    Slow,
    #[serde(rename = "V")]
    Vapor,
}

serde_plain::derive_display_from_serialize!(PulmonaryAbsorptionType);

/// Clearance class of respiratory tract model of ICRP 30
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClearanceClass {
    #[serde(rename = "D")]
    Day,
    #[serde(rename = "W")]
    Week,
    #[serde(rename = "Y")]
    Year,
    #[serde(rename = "V")]
    WaterVapor,
    #[serde(rename = "c")]
    LabelledOrganicCompounds,
    #[serde(rename = "m")]
    CarbonMonoxide,
    #[serde(rename = "d")]
    CarbonDioxide,
}

serde_plain::derive_display_from_serialize!(ClearanceClass);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgeGroup {
    /// From 0 to 1-year (Public)
    #[serde(rename = "Newborn")]
    ThreeMonth,

    /// From 1 to 2-year (Public)
    #[serde(rename = "1 yr-old")]
    OneYear,

    /// More than 2 to 7-year (Public)
    #[serde(rename = "5 yr-old")]
    FiveYear,

    /// More than 7 to 12-year (Public)
    #[serde(rename = "10 yr-old")]
    TenYear,

    /// More than 12 to 17-year (Public)
    #[serde(rename = "15 yr-old")]
    FifteenYear,

    /// More than 17-year (Public)
    Adult,

    /// Radiation worker
    Worker,
}

serde_plain::derive_display_from_serialize!(AgeGroup);
serde_plain::derive_fromstr_from_deserialize!(AgeGroup, |e| -> Error {
    Error::InvalidAgeGroup(e.to_string())
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Organ {
    Adrenals,
    UrinaryBladder,
    BoneSurface,
    Brain,
    Breast,
    Esophagus,
    Stomach,
    SmallIntestine,
    UpperLargeIntestine,
    LowerLargeIntestine,
    Colon,
    Kidneys,
    Liver,
    Muscle,
    Ovaries,
    Pancreas,
    RedMarrow,
    ExtrathoracicAirways,
    Lungs,
    Skin,
    Spleen,
    Testes,
    Thymus,
    Thyroid,
    Uterus,
    Remainder,
    EffectiveDose,
    EffectiveDoseEquivalent,
}

serde_plain::derive_display_from_serialize!(Organ);
serde_plain::derive_fromstr_from_deserialize!(Organ, |e| -> Error {
    Error::InvalidOrgan(e.to_string())
});

/// Dose conversion factor value
#[derive(Debug, PartialEq)]
pub struct DcfValue {
    pub value: f64,
    pub unit: String,
    pub attr: Option<BiokineticAttr>,
}
