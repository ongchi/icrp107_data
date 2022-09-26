use std::path::Path;

use chumsky::Parser;
use mdbsql::Connection;

use crate::error::Error;
use crate::primitive::attr::{IngestionDoseCoefficient, InhalationDoseCoefficient};
use crate::primitive::dose_coefficient::{
    AgeGroup, IngestionDoseCoefficientValue, InhalationDoseCoefficientValue, Organ,
};
use crate::primitive::parser::gi_absorption_factor;
use crate::primitive::Nuclide;

pub struct RadtoolsIcrp68 {
    connection: Connection,
}

impl RadtoolsIcrp68 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            connection: Connection::open(path)?,
        })
    }
}

impl IngestionDoseCoefficient for RadtoolsIcrp68 {
    fn ingestion_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IngestionDoseCoefficientValue>, Error> {
        match age_group {
            AgeGroup::Worker => {
                let rows = self.connection.prepare(&format!(
                    "SELECT {}, f1 FROM Ingestion WHERE Nuclide='{}'",
                    organ, nuclide
                ))?;

                let mut res = vec![];
                for row in rows {
                    let value = row.get(0)?;
                    let (f1, compound) = gi_absorption_factor().parse(row.get::<String>(1)?)?;
                    res.push(IngestionDoseCoefficientValue {
                        value,
                        f1,
                        compound,
                    })
                }

                Ok(res)
            }
            _ => Ok(vec![]),
        }
    }
}

impl InhalationDoseCoefficient for RadtoolsIcrp68 {
    fn inhalation_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<crate::primitive::dose_coefficient::InhalationDoseCoefficientValue>, Error>
    {
        match age_group {
            AgeGroup::Worker => {
                let rows = self.connection.prepare(&format!(
                    "SELECT {}, Type, f1 FROM Inhalation WHERE Nuclide='{}'",
                    organ, nuclide
                ))?;

                let mut res = vec![];
                for row in rows {
                    let value = row.get(0)?;
                    let absorption_type = row.get(1)?;
                    let (f1, compound) = gi_absorption_factor().parse(row.get::<String>(2)?)?;
                    res.push(InhalationDoseCoefficientValue {
                        value,
                        absorption_type,
                        f1,
                        compound,
                    })
                }

                Ok(res)
            }
            _ => Ok(vec![]),
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
        let db = RadtoolsIcrp68::open(DATA_PATH).unwrap();
        let results = db
            .ingestion_dose_coefficients("H-3".parse().unwrap(), AgeGroup::Worker, Organ::Effective)
            .unwrap();

        assert_eq!(
            results,
            vec![
                IngestionDoseCoefficientValue {
                    value: 4.2e-11,
                    f1: 1.0,
                    compound: "OBT".to_string(),
                },
                IngestionDoseCoefficientValue {
                    value: 1.8e-11,
                    f1: 1.0,
                    compound: "".to_string()
                }
            ]
        );
    }

    #[test]
    #[ignore]
    fn inhalation_h3() {
        let db = RadtoolsIcrp68::open(DATA_PATH).unwrap();
        let results = db
            .inhalation_dose_coefficients(
                "H-3".parse().unwrap(),
                AgeGroup::Worker,
                Organ::Effective,
            )
            .unwrap();

        assert_eq!(
            results,
            vec![
                InhalationDoseCoefficientValue {
                    value: 4.1e-11,
                    absorption_type: PulmonaryAbsorptionType::Vapour,
                    f1: 1.,
                    compound: "OBT".to_string()
                },
                InhalationDoseCoefficientValue {
                    value: 1.8e-15,
                    absorption_type: PulmonaryAbsorptionType::Vapour,
                    f1: 1.,
                    compound: "HT".to_string()
                },
                InhalationDoseCoefficientValue {
                    value: 1.8e-13,
                    absorption_type: PulmonaryAbsorptionType::Vapour,
                    f1: 1.,
                    compound: "CH3T".to_string()
                },
                InhalationDoseCoefficientValue {
                    value: 1.8e-11,
                    absorption_type: PulmonaryAbsorptionType::Vapour,
                    f1: 1.,
                    compound: "HTO".to_string()
                },
            ]
        );
    }
}
