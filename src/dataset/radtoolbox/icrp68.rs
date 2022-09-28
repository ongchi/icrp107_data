use std::path::Path;

use chumsky::Parser;
use mdbsql::Connection;

use crate::dataset::radtoolbox::utils::AsAgeDepPhantomOrgan;
use crate::error::Error;
use crate::primitive::attr::{IngestionDoseCoefficient, InhalationDoseCoefficient};
use crate::primitive::dose_coefficient::{AgeGroup, BiokineticAttr, IntExpDcf, Organ};
use crate::primitive::parser::gi_absorption_factor;
use crate::primitive::Nuclide;

pub struct Icrp68 {
    connection: Connection,
}

impl Icrp68 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            connection: Connection::open(path)?,
        })
    }
}

impl IngestionDoseCoefficient for Icrp68 {
    fn ingestion_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error> {
        match age_group {
            AgeGroup::Adult => {
                let rows = self.connection.prepare(&format!(
                    "SELECT {}, f1 FROM Ingestion WHERE Nuclide='{}'",
                    organ.to_col()?,
                    nuclide
                ))?;

                let mut res = vec![];
                for row in rows {
                    let value = row.get(0)?;
                    let (f1, compound) = gi_absorption_factor().parse(row.get::<String>(1)?)?;
                    res.push(IntExpDcf {
                        value,
                        bio_attr: BiokineticAttr {
                            f1,
                            compound,
                            ..Default::default()
                        },
                    })
                }

                Ok(res)
            }
            _ => Err(Error::InvalidAgeGroup(age_group)),
        }
    }
}

impl InhalationDoseCoefficient for Icrp68 {
    fn inhalation_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error> {
        match age_group {
            AgeGroup::Adult => {
                let rows = self.connection.prepare(&format!(
                    "SELECT {}, Type, f1 FROM Inhalation WHERE Nuclide='{}'",
                    organ.to_col()?,
                    nuclide
                ))?;

                let mut res = vec![];
                for row in rows {
                    let value = row.get(0)?;
                    let pulmonary_absorption_type = row.get(1)?;
                    let (f1, compound) = gi_absorption_factor().parse(row.get::<String>(2)?)?;
                    res.push(IntExpDcf {
                        value,
                        bio_attr: BiokineticAttr {
                            f1,
                            compound,
                            pulmonary_absorption_type,
                            ..Default::default()
                        },
                    })
                }

                Ok(res)
            }
            _ => Err(Error::InvalidAgeGroup(age_group)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitive::dose_coefficient::PulmonaryAbsorptionType;

    const DATA_PATH: &str = "data/RadToolbox3/icrp68.mdb";

    #[test]
    #[ignore]
    fn ingestion_h3() {
        let db = Icrp68::open(DATA_PATH).unwrap();
        let results = db
            .ingestion_dose_coefficients(
                "H-3".parse().unwrap(),
                AgeGroup::Adult,
                Organ::EffectiveDose,
            )
            .unwrap();

        assert_eq!(
            results,
            vec![
                IntExpDcf {
                    value: 4.2e-11,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        compound: "OBT".to_string(),
                        ..Default::default()
                    }
                },
                IntExpDcf {
                    value: 1.8e-11,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        ..Default::default()
                    }
                }
            ]
        );
    }

    #[test]
    #[ignore]
    fn inhalation_h3() {
        let db = Icrp68::open(DATA_PATH).unwrap();
        let results = db
            .inhalation_dose_coefficients(
                "H-3".parse().unwrap(),
                AgeGroup::Adult,
                Organ::EffectiveDose,
            )
            .unwrap();

        assert_eq!(
            results,
            vec![
                IntExpDcf {
                    value: 4.1e-11,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        compound: "OBT".to_string(),
                        pulmonary_absorption_type: Some(PulmonaryAbsorptionType::Vapor),
                        ..Default::default()
                    }
                },
                IntExpDcf {
                    value: 1.8e-15,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        compound: "HT".to_string(),
                        pulmonary_absorption_type: Some(PulmonaryAbsorptionType::Vapor),
                        ..Default::default()
                    }
                },
                IntExpDcf {
                    value: 1.8e-13,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        compound: "CH3T".to_string(),
                        pulmonary_absorption_type: Some(PulmonaryAbsorptionType::Vapor),
                        ..Default::default()
                    }
                },
                IntExpDcf {
                    value: 1.8e-11,
                    bio_attr: BiokineticAttr {
                        f1: 1.,
                        compound: "HTO".to_string(),
                        pulmonary_absorption_type: Some(PulmonaryAbsorptionType::Vapor),
                        ..Default::default()
                    }
                },
            ]
        );
    }
}
