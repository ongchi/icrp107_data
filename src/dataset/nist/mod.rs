mod reader;

use std::path::Path;
use std::{collections::BTreeMap, path::PathBuf};

use fixed_width_derive::FixedWidth;
use num_traits::FromPrimitive;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::error::Error;
use crate::nuclide::Symbol;
use reader::{MassAttenCoefReader, MaterialConstantReader};

/// Energy in keV
pub type Energy = u32;

static MATEAIAL_CONSTANTS: OnceCell<BTreeMap<Symbol, MaterialConstant>> = OnceCell::new();
static ATTENUATION_COEF: OnceCell<BTreeMap<Symbol, BTreeMap<Energy, MassAttenCoef>>> =
    OnceCell::new();

#[derive(Deserialize, FixedWidth)]
pub struct MaterialConstantRecord {
    #[fixed_width(range = "4..6")]
    symbol: Symbol,

    /// Z/A
    #[fixed_width(range = "30..40")]
    z_over_a: f64,

    /// I (eV)
    #[fixed_width(range = "40..50")]
    i: f64,

    /// Density (g/cm3)
    #[fixed_width(range = "50..60")]
    density: f64,
}

pub struct MaterialConstant {
    /// Z/A
    pub z_over_a: f64,
    /// I (eV)
    pub i: f64,
    /// Density (g/cm3)
    pub density: f64,
}

impl From<MaterialConstantRecord> for MaterialConstant {
    fn from(record: MaterialConstantRecord) -> Self {
        Self {
            z_over_a: record.z_over_a,
            i: record.i,
            density: record.density,
        }
    }
}

#[derive(Deserialize, FixedWidth)]
pub struct MassAttenCoefRecord {
    /// Energy (MeV)
    #[fixed_width(range = "0..12")]
    energy: f64,

    /// mu/rho (cm2/g)
    #[fixed_width(range = "12..24")]
    mu_over_rho: f64,

    /// mu_en/rho (cm2/g)
    #[fixed_width(range = "24..36")]
    mu_en_over_rho: f64,
}

pub struct MassAttenCoef {
    pub mu_over_rho: f64,
    pub mu_en_over_rho: f64,
}

impl From<MassAttenCoefRecord> for MassAttenCoef {
    fn from(record: MassAttenCoefRecord) -> Self {
        Self {
            mu_over_rho: record.mu_over_rho,
            mu_en_over_rho: record.mu_en_over_rho,
        }
    }
}

pub struct NistMassAttenCoef {
    path: PathBuf,
}

impl NistMassAttenCoef {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn material_constants(&self) -> Result<&BTreeMap<Symbol, MaterialConstant>, Error> {
        MATEAIAL_CONSTANTS.get_or_try_init(|| {
            MaterialConstantReader::new(&self.path.join("material_constants")).read()
        })
    }

    pub fn mass_atten_coef(
        &self,
    ) -> Result<&BTreeMap<Symbol, BTreeMap<Energy, MassAttenCoef>>, Error> {
        ATTENUATION_COEF.get_or_try_init(|| {
            let mut content = BTreeMap::new();

            for z in 1..=92 {
                let symbol: Symbol = FromPrimitive::from_u8(z).unwrap();
                let value = MassAttenCoefReader::new(&self.path, z)
                    .read()?
                    .into_iter()
                    .map(|r| ((r.energy * 1000.) as u32, r.into()))
                    .collect();

                content.insert(symbol, value);
            }

            Ok(content)
        })
    }
}
