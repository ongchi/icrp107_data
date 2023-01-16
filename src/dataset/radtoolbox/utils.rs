use crate::error::Error;
use crate::primitive::dose_coefficient::Organ;

/// Tissues and organs for dose coefficients (FGR12)
pub trait AsAdultPhantomOrgan {
    fn to_col(self) -> Result<String, Error>;
}

impl AsAdultPhantomOrgan for Organ {
    fn to_col(self) -> Result<String, Error> {
        match self {
            Organ::Adrenals => Ok("Adrenals".to_string()),
            Organ::UrinaryBladder => Ok("Bladder Wall".to_string()),
            Organ::BoneSurface => Ok("Bone Surface".to_string()),
            Organ::Brain => Ok("Brain".to_string()),
            Organ::Breast => Ok("Breast".to_string()),
            Organ::Esophagus => Ok("Esophagus".to_string()),
            Organ::Stomach => Ok("Stomach Wall".to_string()),
            Organ::SmallIntestine => Ok("Small Intestine Wall".to_string()),
            Organ::UpperLargeIntestine => Ok("Upper Large IntestineWall".to_string()),
            Organ::LowerLargeIntestine => Ok("Lower Large IntestineWall".to_string()),
            Organ::Colon => Err(Error::InvalidOrgan(self.to_string())),
            Organ::Kidneys => Ok("Kidneys".to_string()),
            Organ::Liver => Ok("Liver".to_string()),
            Organ::Muscle => Ok("Muscle".to_string()),
            Organ::Ovaries => Ok("Ovaries".to_string()),
            Organ::Pancreas => Ok("Pancreas".to_string()),
            Organ::RedMarrow => Ok("Red Marrow".to_string()),
            Organ::ExtrathoracicAirways => Err(Error::InvalidOrgan(self.to_string())),
            Organ::Lungs => Ok("Lungs".to_string()),
            Organ::Skin => Ok("Skin".to_string()),
            Organ::Spleen => Ok("Spleen".to_string()),
            Organ::Testes => Ok("Testes".to_string()),
            Organ::Thymus => Ok("Thymus".to_string()),
            Organ::Thyroid => Ok("Thyroid".to_string()),
            Organ::Uterus => Ok("Uterus".to_string()),
            Organ::Remainder => Err(Error::InvalidOrgan(self.to_string())),
            Organ::EffectiveDose => Ok("E".to_string()),
            Organ::EffectiveDoseEquivalent => Ok("h E".to_string()),
        }
    }
}

/// Tissues and organs for age-dependent dose coefficients (ICRP68/72)
pub trait AsAgeDepPhantomOrgan {
    fn to_col(self) -> Result<String, Error>;
}

impl AsAgeDepPhantomOrgan for Organ {
    fn to_col(self) -> Result<String, Error> {
        match self {
            Self::Adrenals => Ok("Adrenals".to_string()),
            Self::UrinaryBladder => Ok("Urinary Bladder".to_string()),
            Self::BoneSurface => Ok("Bone Surface".to_string()),
            Self::Brain => Ok("Brain".to_string()),
            Self::Breast => Ok("Breast".to_string()),
            Self::Esophagus => Ok("Esophagus".to_string()),
            Self::Stomach => Ok("Stomach".to_string()),
            Self::SmallIntestine => Ok("Small Intestine".to_string()),
            Self::UpperLargeIntestine => Ok("Upper Large Intestine".to_string()),
            Self::LowerLargeIntestine => Ok("Lower Large Intestine".to_string()),
            Self::Colon => Ok("Colon".to_string()),
            Self::Kidneys => Ok("Kidneys".to_string()),
            Self::Liver => Ok("Liver".to_string()),
            Self::Muscle => Ok("Muscle".to_string()),
            Self::Ovaries => Ok("Ovaries".to_string()),
            Self::Pancreas => Ok("Pancreas".to_string()),
            Self::RedMarrow => Ok("Red Marrow".to_string()),
            Self::ExtrathoracicAirways => Ok("Extrathoracic Airways".to_string()),
            Self::Lungs => Ok("Lungs".to_string()),
            Self::Skin => Ok("Skin".to_string()),
            Self::Spleen => Ok("Spleen".to_string()),
            Self::Testes => Ok("Testes".to_string()),
            Self::Thymus => Ok("Thymus".to_string()),
            Self::Thyroid => Ok("Thyroid".to_string()),
            Self::Uterus => Ok("Uterus".to_string()),
            Self::Remainder => Ok("Remainder".to_string()),
            Self::EffectiveDose => Ok("E".to_string()),
            Self::EffectiveDoseEquivalent => Err(Error::InvalidOrgan(self.to_string())),
        }
    }
}
