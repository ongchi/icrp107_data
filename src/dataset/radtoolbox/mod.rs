use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::OnceCell;

use crate::error::Error;
use crate::primitive::attr::{IngestionDoseCoefficient, InhalationDoseCoefficient};
use crate::primitive::dose_coefficient::AgeGroup;

pub mod icrp68;
pub mod icrp72;

pub struct RadToolbox3 {
    root_path: PathBuf,
    icrp68: OnceCell<icrp68::RadtoolsIcrp68>,
    icrp72: OnceCell<icrp72::RadtoolsIcrp72>,
}

impl RadToolbox3 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Arc<Self>, Error> {
        let root_path = path.as_ref().to_path_buf();
        if root_path.is_dir() {
            Ok(Arc::new(Self {
                root_path,
                icrp68: OnceCell::new(),
                icrp72: OnceCell::new(),
            }))
        } else {
            Err(Error::Unexpected(anyhow::anyhow!("Invalid data path")))
        }
    }
}

impl IngestionDoseCoefficient for RadToolbox3 {
    fn ingestion_dose_coefficients(
        &self,
        nuclide: crate::primitive::Nuclide,
        age_group: crate::primitive::dose_coefficient::AgeGroup,
        organ: crate::primitive::dose_coefficient::Organ,
    ) -> Result<Vec<crate::primitive::dose_coefficient::IngestionDoseCoefficientValue>, Error> {
        let data = match age_group {
            AgeGroup::Worker => self.icrp68.get_or_try_init(|| {
                icrp68::RadtoolsIcrp68::open(self.root_path.join("icrp68.mdb"))
            })? as &dyn IngestionDoseCoefficient,
            _ => self.icrp72.get_or_try_init(|| {
                icrp72::RadtoolsIcrp72::open(self.root_path.join("icrp72.mdb"))
            })? as &dyn IngestionDoseCoefficient,
        };

        data.ingestion_dose_coefficients(nuclide, age_group, organ)
    }
}

impl InhalationDoseCoefficient for RadToolbox3 {
    fn inhalation_dose_coefficients(
        &self,
        nuclide: crate::primitive::Nuclide,
        age_group: AgeGroup,
        organ: crate::primitive::dose_coefficient::Organ,
    ) -> Result<Vec<crate::primitive::dose_coefficient::InhalationDoseCoefficientValue>, Error>
    {
        let data = match age_group {
            AgeGroup::Worker => self.icrp68.get_or_try_init(|| {
                icrp68::RadtoolsIcrp68::open(self.root_path.join("icrp68.mdb"))
            })? as &dyn InhalationDoseCoefficient,
            _ => self.icrp72.get_or_try_init(|| {
                icrp72::RadtoolsIcrp72::open(self.root_path.join("icrp72.mdb"))
            })? as &dyn InhalationDoseCoefficient,
        };

        data.inhalation_dose_coefficients(nuclide, age_group, organ)
    }
}
