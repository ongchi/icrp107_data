use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub enum AgeGroup {
    // From 0 to 1-year (Public)
    #[serde(rename = "Newborn")]
    ThreeMonth,

    // From 1 to 2-year (Public)
    #[serde(rename = "1 yr-old")]
    OneYear,

    // More than 2 to 7-year (Public)
    #[serde(rename = "5 yr-old")]
    FiveYear,

    // More than 7 to 12-year (Public)
    #[serde(rename = "10 yr-old")]
    TenYear,

    // More than 12 to 17-year (Public)
    #[serde(rename = "15 yr-old")]
    FifteenYear,

    // More than 17-year (Public)
    Adult,

    // Radiation worker
    Worker,
}

serde_plain::derive_display_from_serialize!(AgeGroup);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PulmonaryAbsorptionType {
    #[serde(rename = "F")]
    Fast,
    #[serde(rename = "M")]
    Moderate,
    #[serde(rename = "S")]
    Slow,
    #[serde(rename = "V")]
    Vapour,
}

serde_plain::derive_display_from_serialize!(PulmonaryAbsorptionType);

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Organ {
    Adrenals,
    #[serde(rename = "Urinary Bladder")]
    UrinaryBladder,
    #[serde(rename = "Bone Surface")]
    BoneSurface,
    Brain,
    Breast,
    Esophagus,
    Stomach,
    SmallIntestine,
    #[serde(rename = "Upper Large Intestine")]
    UpperLargeIntestine,
    #[serde(rename = "Lower Large Intestine")]
    LowerLargeIntestine,
    Colon,
    Kidneys,
    Liver,
    Muscle,
    Ovaries,
    Pancreas,
    #[serde(rename = "Red Marrow")]
    RedMarrow,
    #[serde(rename = "Extrathoracic Airways")]
    ExtrathoracicAirways,
    Lungs,
    Skin,
    Spleen,
    Testes,
    Thymus,
    Thyroid,
    Uterus,
    Remainder,
    #[serde(rename = "E")]
    Effective,
}

serde_plain::derive_display_from_serialize!(Organ);

#[derive(Debug, PartialEq)]
pub struct IngestionDoseCoefficientValue {
    // Sv/Bq
    pub value: f64,
    // Gastroinstestinal absorption factor
    pub f1: f64,
    pub compound: String,
}

#[derive(Debug, PartialEq)]
pub struct InhalationDoseCoefficientValue {
    // Sv/Bq
    pub value: f64,
    pub absorption_type: PulmonaryAbsorptionType,
    // Gastroinstestinal absorption factor
    pub f1: f64,
    pub compound: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn organ_to_string() {
        assert_eq!(Organ::Effective.to_string(), "E");
        assert_eq!(
            Organ::UpperLargeIntestine.to_string(),
            "Upper Large Intestine"
        );
    }
}
