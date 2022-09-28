use std::collections::BTreeMap;

use super::dose_coefficient::{AgeGroup, IntExpDcf, Organ};
use super::notation::{Material, Symbol};
use super::nuclide::{HalfLife, Nuclide, Progeny};
use crate::error::Error;

/// Energy in eV
pub type Energy = u32;

pub trait NuclideProgeny {
    fn progeny(&self, nuclide: Nuclide) -> Result<Vec<Progeny>, Error>;
}

pub trait NuclideHalfLife {
    // Half life (arbitrary unit)
    fn half_life(&self, nuclide: Nuclide) -> Result<HalfLife, Error>;
}

pub trait DecayConstant {
    // Decay constant (s-1)
    fn lambda(&self, nuclide: Nuclide) -> Result<f64, Error>;
}

impl<T> DecayConstant for T
where
    T: NuclideHalfLife,
{
    fn lambda(&self, nuclide: Nuclide) -> Result<f64, Error> {
        self.half_life(nuclide).map(|hl| hl.as_lambda())
    }
}

pub trait Atom {
    fn symbol(&self) -> Symbol;
    fn nuclide(&self) -> Nuclide;
}

pub trait AtomicMass {
    /// Atomic mass (amu)
    fn atomic_mass(&self, symbol: Symbol) -> Result<f64, Error>;
}

pub trait MassAttenuationCoefficient {
    /// Mass attenuation coefficient (cm2/g)
    fn mass_attenuation_coefficient(
        &self,
        material: &Material,
        energy: Energy,
    ) -> Result<f64, Error>;
}

pub trait MeanFreePath {
    /// Mean free path (cm)
    fn mfp(&self, material: &Material, energy: Energy) -> Result<f64, Error>;
}

impl<T> MeanFreePath for T
where
    T: MassAttenuationCoefficient,
{
    fn mfp(&self, material: &Material, energy: Energy) -> Result<f64, Error> {
        self.mass_attenuation_coefficient(material, energy)
            .map(|mu_over_rho| (mu_over_rho * material.density()).recip())
    }
}

pub trait EffectiveAtomicNumber {
    // Effective atomic number
    fn z_eff(&self) -> Result<f64, Error>;
}

// Effective atomic number
pub fn z_eff(composition: &BTreeMap<Symbol, f64>) -> f64 {
    let mut composition = composition.clone();
    let mut tot_n = 0f64;

    // composition normalization
    for (symbol, n) in &mut composition {
        let z = (*symbol as u8) as f64;
        *n *= z;
    }

    for n in composition.values() {
        tot_n += n;
    }

    for n in composition.values_mut() {
        *n /= tot_n
    }

    let mut zeff = 0f64;

    for (&symbol, &f) in &composition {
        let z = (symbol as u8) as f64;
        zeff += f * z.powf(2.94);
    }

    zeff.powf(2.94f64.recip())
}

pub trait AirSubmersionDoseCoefficient {
    fn air_submersion_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait WaterSubmersionDoseCoefficient {
    fn water_submersion_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait GroundSurfaceDoseCoefficient {
    fn ground_surface_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait SoilOneCmDoseCoefficient {
    fn soil_1cm_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait SoilFiveCmDoseCoefficient {
    fn soil_5cm_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait SoilFifteenCmDoseCoefficient {
    fn soil_15cm_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait SoilInfiniteDoseCoefficient {
    fn soil_infinite_dose_coefficient(
        &self,
        nuclide: Nuclide,
        organ: Organ,
    ) -> Result<Option<f64>, Error>;
}

pub trait IngestionDoseCoefficient {
    fn ingestion_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error>;
}

pub trait InhalationDoseCoefficient {
    fn inhalation_dose_coefficients(
        &self,
        nuclide: Nuclide,
        age_group: AgeGroup,
        organ: Organ,
    ) -> Result<Vec<IntExpDcf>, Error>;
}
