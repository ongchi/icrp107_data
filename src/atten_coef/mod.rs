use std::collections::BTreeMap;

use crate::error::Error;
use crate::molecular::Molecular;
use crate::nuclide::Symbol;

/// Energy in eV
pub type Energy = u32;

// /// Mean free path (cm)
// pub type MFP = f64;

pub trait AttenCoefData {
    /// Atomic mass (amu)
    fn mass_number(&self, symbol: Symbol) -> Result<f64, Error>;

    /// Mass attenuation coefficient (cm2/g)
    fn mass_attenuation_coefficient(
        &self,
        material: &Material,
        energy: Energy,
    ) -> Result<f64, Error>;

    /// Mean free path (cm)
    fn mfp(&self, material: &Material, energy: Energy) -> Result<f64, Error>;
}

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

pub struct MaterialBuilder<'a> {
    data: &'a dyn AttenCoefData,
    composition: BTreeMap<Symbol, f64>,
    weight_fraction: BTreeMap<Symbol, f64>,
    density: Option<f64>,
    weight: Option<f64>,
}

impl<'a> MaterialBuilder<'a> {
    pub fn new(data: &'a dyn AttenCoefData) -> Self {
        Self {
            data,
            composition: BTreeMap::new(),
            weight_fraction: BTreeMap::new(),
            density: None,
            weight: None,
        }
    }

    pub fn formula(mut self, formula: &str) -> Result<Self, Error> {
        let molecular = formula
            .parse::<Molecular>()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let composition: BTreeMap<Symbol, f64> = molecular
            .composition()
            .into_iter()
            .map(|(k, v)| (k, v as f64))
            .collect();

        let mut weight_fraction = BTreeMap::new();
        let mut tot = 0f64;

        for (&symbol, &n) in &composition {
            let mass = n * self.data.mass_number(symbol)?;
            tot += mass;
            weight_fraction.insert(symbol, mass);
        }

        for f in weight_fraction.values_mut() {
            *f /= tot;
        }

        self.composition = composition;
        self.weight_fraction = weight_fraction;

        Ok(self)
    }

    pub fn weights(mut self, weights: BTreeMap<Symbol, f64>) -> Result<Self, Error> {
        let mut weight_fraction = weights;
        let mut tot = 0f64;

        for w in weight_fraction.values() {
            tot += w;
        }

        for w in weight_fraction.values_mut() {
            *w /= tot;
        }

        self = self.weight_fraction(weight_fraction)?;
        self.weight = Some(tot);

        Ok(self)
    }

    pub fn weight_fraction(
        mut self,
        weight_fraction: BTreeMap<Symbol, f64>,
    ) -> Result<Self, Error> {
        let mut composition = BTreeMap::new();

        for (&symbol, &f) in &weight_fraction {
            let n = f / self.data.mass_number(symbol)?;
            composition.insert(symbol, n);
        }

        self.weight_fraction = weight_fraction;
        self.composition = composition;

        Ok(self)
    }

    pub fn density(mut self, density: f64) -> Self {
        self.density = Some(density);

        self
    }

    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);

        self
    }

    pub fn build(self) -> Result<Material, Error> {
        if self.weight.is_some() && self.density.is_some() {
            Ok(Material {
                composition: self.composition,
                weight_fraction: self.weight_fraction,
                density: self.density.unwrap(),
                weight: self.weight.unwrap(),
            })
        } else {
            Err(anyhow::anyhow!("Data incomplete").into())
        }
    }
}

#[derive(Debug)]
pub struct Material {
    composition: BTreeMap<Symbol, f64>,
    weight_fraction: BTreeMap<Symbol, f64>,
    density: f64,
    weight: f64,
}

impl Material {
    pub fn composition(&self) -> &BTreeMap<Symbol, f64> {
        &self.composition
    }

    pub fn weight_fraction(&self) -> &BTreeMap<Symbol, f64> {
        &self.weight_fraction
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    /// Effective atomic number
    pub fn z_eff(&self) -> f64 {
        z_eff(self.composition())
    }
}
