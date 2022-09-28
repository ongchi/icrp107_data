use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq)]
pub struct BiokineticAttr {
    /// Gastroinstestinal absorption factor
    pub f1: f64,
    /// Chemical compound
    pub compound: String,
    /// Pulmonary Absorption Type of respiratory tract model of ICRP 66
    pub pulmonary_absorption_type: Option<PulmonaryAbsorptionType>,
    /// Clearance class of respiratory tract model of ICRP 30
    pub clearance_class: Option<ClearanceClass>,
}

/// Pulmonary Absorption Type of respiratory tract model of ICRP 66
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize, PartialEq, Eq)]
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

/// Dose conversion factor of internal exposure (Sv/Bq)
#[derive(Debug, PartialEq)]
pub struct IntExpDcf {
    pub value: f64,
    pub bio_attr: BiokineticAttr,
}
