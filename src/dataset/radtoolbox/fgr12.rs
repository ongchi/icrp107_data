use std::path::Path;

use chumsky::Parser;
use mdbsql::Connection;

use crate::dataset::radtoolbox::utils::AsAdultPhantomOrgan;
use crate::error::Error;
use crate::primitive::dose_coefficient::{AgeGroup, BiokineticAttr, IntExpDcf, Organ};
use crate::primitive::parser::gi_absorption_factor;
use crate::primitive::{
    AirSubmersionDoseCoefficient, GroundSurfaceDoseCoefficient, IngestionDoseCoefficient,
    InhalationDoseCoefficient, Nuclide, SoilFifteenCmDoseCoefficient, SoilFiveCmDoseCoefficient,
    SoilInfiniteDoseCoefficient, SoilOneCmDoseCoefficient, WaterSubmersionDoseCoefficient,
};

#[derive(Debug)]
pub struct Fgr12 {
    connection: Connection,
}

impl Fgr12 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            connection: Connection::open(path.as_ref())?,
        })
    }
}

macro_rules! ext_dcf_fn {
    ($fn:ident, $table:expr) => {
        fn $fn(&self, nuclide: Nuclide, organ: Organ) -> Result<Option<f64>, Error> {
            if let Some(row) = self
                .connection
                .prepare(&format!(
                    concat!("SELECT \"{}\" FROM \"", $table, "\" WHERE Nuclide='{}'"),
                    organ.to_col()?,
                    nuclide
                ))?
                .next()
            {
                Ok(Some(row.get(0)?))
            } else {
                Ok(None)
            }
        }
    };
}

impl AirSubmersionDoseCoefficient for Fgr12 {
    ext_dcf_fn!(air_submersion_dose_coefficient, "Air Submersion");
}

impl WaterSubmersionDoseCoefficient for Fgr12 {
    ext_dcf_fn!(water_submersion_dose_coefficient, "Water Submersion");
}

impl GroundSurfaceDoseCoefficient for Fgr12 {
    ext_dcf_fn!(ground_surface_dose_coefficient, "Ground Surface");
}

impl SoilOneCmDoseCoefficient for Fgr12 {
    ext_dcf_fn!(soil_1cm_dose_coefficient, "1 cm Soil");
}

impl SoilFiveCmDoseCoefficient for Fgr12 {
    ext_dcf_fn!(soil_5cm_dose_coefficient, "5 cm Soil");
}

impl SoilFifteenCmDoseCoefficient for Fgr12 {
    ext_dcf_fn!(soil_15cm_dose_coefficient, "15 cm Soil");
}

impl SoilInfiniteDoseCoefficient for Fgr12 {
    ext_dcf_fn!(soil_infinite_dose_coefficient, "Infinite Soil");
}

impl IngestionDoseCoefficient for Fgr12 {
    fn ingestion_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error> {
        match age_group {
            AgeGroup::Worker => {
                let rows = self.connection.prepare(&format!(
                    "SELECT \"{}\", f1 FROM Ingestion WHERE Nuclide='{}'",
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
            _ => Ok(vec![]),
        }
    }
}

impl InhalationDoseCoefficient for Fgr12 {
    fn inhalation_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error> {
        match age_group {
            AgeGroup::Worker => {
                let rows = self.connection.prepare(&format!(
                    "SELECT \"{}\", Class, f1 FROM Inhalation WHERE Nuclide='{}'",
                    organ.to_col()?,
                    nuclide
                ))?;

                let mut res = vec![];
                for row in rows {
                    let value = row.get(0)?;
                    let clearance_class = row.get(1)?;
                    let (f1, compound) = gi_absorption_factor().parse(row.get::<String>(2)?)?;
                    res.push(IntExpDcf {
                        value,
                        bio_attr: BiokineticAttr {
                            f1,
                            compound,
                            clearance_class,
                            ..Default::default()
                        },
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
    use crate::primitive::dose_coefficient::ClearanceClass;

    const DATA_PATH: &str = "data/RadToolbox3/fgr12.mdb";

    #[test]
    #[ignore]
    fn air_submersion_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .air_submersion_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(9.28e-17));
    }

    #[test]
    #[ignore]
    fn water_submersion_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .water_submersion_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(1.04e-19));
    }

    #[test]
    #[ignore]
    fn ground_surface_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .ground_surface_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(2.99e-18));
    }

    #[test]
    #[ignore]
    fn soil_1cm_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .soil_1cm_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(2.07e-21));
    }

    #[test]
    #[ignore]
    fn soil_5cm_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .soil_5cm_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(3.62e-21));
    }

    #[test]
    #[ignore]
    fn soil_15cm_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .soil_15cm_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(4.4e-21));
    }

    #[test]
    #[ignore]
    fn soil_infinite_cs137() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let result = db
            .soil_infinite_dose_coefficient("Cs-137".parse().unwrap(), Organ::EffectiveDose)
            .unwrap();

        assert_eq!(result, Some(4.47e-21));
    }

    #[test]
    #[ignore]
    fn ingestion_h3() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let results = db
            .ingestion_dose_coefficients(
                "H-3".parse().unwrap(),
                AgeGroup::Worker,
                Organ::EffectiveDoseEquivalent,
            )
            .unwrap();

        assert_eq!(
            results,
            vec![IntExpDcf {
                value: 1.73e-11,
                bio_attr: BiokineticAttr {
                    f1: 1.,
                    ..Default::default()
                },
            },]
        );
    }

    #[test]
    #[ignore]
    fn inhalation_h3() {
        let db = Fgr12::open(DATA_PATH).unwrap();
        let results = db
            .inhalation_dose_coefficients(
                "H-3".parse().unwrap(),
                AgeGroup::Worker,
                Organ::EffectiveDoseEquivalent,
            )
            .unwrap();

        assert_eq!(
            results,
            vec![IntExpDcf {
                value: 1.73e-11,
                bio_attr: BiokineticAttr {
                    f1: 1.,
                    clearance_class: Some(ClearanceClass::WaterVapor),
                    ..Default::default()
                },
            },]
        );
    }
}
