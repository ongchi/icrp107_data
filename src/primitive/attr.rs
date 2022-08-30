use std::collections::BTreeMap;

use crate::error::Error;
use crate::primitive::notation::Material;
use crate::primitive::nuclide::{HalfLife, Nuclide, Progeny};
use crate::primitive::Symbol;

/// Energy in eV
pub type Energy = u32;

pub trait NuclideProgeny {
    fn progeny(&self, nuclide: Nuclide) -> Result<&[Progeny], Error>;
}

pub trait NuclideHalfLife {
    // Half life (arbitrary unit)
    fn half_life(&self, nuclide: Nuclide) -> Result<HalfLife, Error>;
}

pub trait DecayConstant {
    // Decay constant (s-1)
    fn lambda(&self, nuclide: Nuclide) -> Result<f64, Error>;
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
