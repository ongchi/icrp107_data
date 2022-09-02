mod ndx;
mod reader;
mod spectrum;

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::Error;
use crate::primitive::attr::{DecayConstant, NuclideHalfLife, NuclideProgeny};
use crate::primitive::{HalfLife, Nuclide, Progeny};
use reader::{IndexReader, SpectrumReader};
use spectrum::{ack, bet, nsf, rad};

static NDX: OnceCell<HashMap<Nuclide, ndx::Attribute>> = OnceCell::new();
static RAD: OnceCell<HashMap<Nuclide, Vec<rad::RadSpectrum>>> = OnceCell::new();
static BET: OnceCell<HashMap<Nuclide, Vec<bet::BetSpectrum>>> = OnceCell::new();
static ACK: OnceCell<HashMap<Nuclide, Vec<ack::AckSpectrum>>> = OnceCell::new();
static NSF: OnceCell<HashMap<Nuclide, Vec<nsf::NsfSpectrum>>> = OnceCell::new();

pub struct Icrp107 {
    path: PathBuf,
}

impl Icrp107 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Arc<Self>, Error> {
        let path = path.as_ref().to_path_buf();

        if path.is_dir() {
            Ok(Arc::new(Self { path }))
        } else {
            Err(Error::Unexpected(anyhow::anyhow!("Invalid data path")))
        }
    }

    pub fn ndx(&self) -> Result<&HashMap<Nuclide, ndx::Attribute>, Error> {
        NDX.get_or_try_init(|| IndexReader::new(&self.path.join("ICRP-07.NDX"))?.read())
    }

    pub fn rad(&self) -> Result<&HashMap<Nuclide, Vec<rad::RadSpectrum>>, Error> {
        RAD.get_or_try_init(|| SpectrumReader::new(&self.path.join("ICRP-07.RAD"))?.read())
    }

    pub fn bet(&self) -> Result<&HashMap<Nuclide, Vec<bet::BetSpectrum>>, Error> {
        BET.get_or_try_init(|| SpectrumReader::new(&self.path.join("ICRP-07.BET"))?.read())
    }

    pub fn ack(&self) -> Result<&HashMap<Nuclide, Vec<ack::AckSpectrum>>, Error> {
        ACK.get_or_try_init(|| SpectrumReader::new(&self.path.join("ICRP-07.ACK"))?.read())
    }

    pub fn nsf(&self) -> Result<&HashMap<Nuclide, Vec<nsf::NsfSpectrum>>, Error> {
        NSF.get_or_try_init(|| SpectrumReader::new(&self.path.join("ICRP-07.NSF"))?.read())
    }
}

impl NuclideProgeny for Icrp107 {
    fn progeny(&self, nuclide: Nuclide) -> Result<&[Progeny], Error> {
        self.ndx()?
            .get(&nuclide)
            .map(|attr| attr.progeny.as_slice())
            .ok_or_else(|| Error::InvalidNuclide(nuclide.to_string()))
    }
}

impl NuclideHalfLife for Icrp107 {
    fn half_life(&self, nuclide: Nuclide) -> Result<HalfLife, Error> {
        self.ndx()?
            .get(&nuclide)
            .map(|attr| attr.half_life)
            .ok_or_else(|| Error::InvalidNuclide(nuclide.to_string()))
    }
}

impl DecayConstant for Icrp107 {
    fn lambda(&self, nuclide: Nuclide) -> Result<f64, Error> {
        self.half_life(nuclide).map(|t| t.as_lambda())
    }
}
