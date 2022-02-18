use fixed_width::{Field, FixedWidth};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::nuclide::{half_life::HalfLife, DecayMode, Nuclide, Symbol};
use super::spectrum::{NuclideSpectrum, Spectrum};
use crate::derive_fixed_width_from_fortran_format;
use crate::error::Error;
use crate::reader::{fields_from_fortran_format, FileReader};
use crate::spectrum::{ack, bet, nsf, rad};

#[derive(Debug, Deserialize)]
struct NdxEntry {
    pub nuclide: Nuclide,
    pub half_life: HalfLife,
    pub decay_mode: DecayMode,
    pub d1: Option<Nuclide>,
    pub d1_branch: Option<f64>,
    pub d2: Option<Nuclide>,
    pub d2_branch: Option<f64>,
    pub d3: Option<Nuclide>,
    pub d3_branch: Option<f64>,
    pub d4: Option<Nuclide>,
    pub d4_branch: Option<f64>,
    pub alpha_energy: f64,
    pub electron_energy: f64,
    pub photon_energy: f64,
    pub n_photon_le_10kev_per_nt: u64,
    pub n_photon_gt_10kev_per_nt: u64,
    pub n_beta_per_nt: u64,
    pub n_mono_electron_per_nt: u64,
    pub n_alpha_per_nt: u64,
    pub amu: f64,
    pub air_kerma_const: f64,
    pub air_kerma_coef: f64,
}

derive_fixed_width_from_fortran_format!(
    NdxEntry,
    "(a7,a10,a8,27x,4(1x,a7,6x,e11.0),1x,f7.0,2f8.0,3i4,i5,i4,e11.0,e10.0,e9.0)"
);

#[derive(Debug)]
pub struct Progeny {
    pub mode: DecayMode,
    pub branch_rate: f64,
    pub nuclide: Nuclide,
}

#[derive(Debug)]
pub struct Attribute {
    pub half_life: HalfLife,
    pub decay_mode: DecayMode,
    pub progeny: Vec<Progeny>,
    pub alpha_energy: f64,
    pub electron_energy: f64,
    pub photon_energy: f64,
    pub n_photon_le_10kev_per_nt: u64,
    pub n_photon_gt_10kev_per_nt: u64,
    pub n_beta_per_nt: u64,
    pub n_mono_electron_per_nt: u64,
    pub n_alpha_per_nt: u64,
    pub amu: f64,
    pub air_kerma_const: f64,
    pub air_kerma_coef: f64,
}

#[derive(Debug)]
pub struct NuclideData {
    pub attribute: HashMap<Nuclide, Attribute>,
    pub spectrum: HashMap<Nuclide, Vec<Spectrum>>,
}

impl NuclideData {
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut ndx = FileReader::new(&path.as_ref().join("ICRP-07.NDX")).skip_lines(1);
        let mut attribute = HashMap::new();
        let mut spectrum: HashMap<Nuclide, Vec<Spectrum>> = HashMap::new();

        let mut buf = String::new();
        while ndx.read_str(&mut buf)? != 0 {
            let row: NdxEntry =
                fixed_width::from_str(&buf).map_err(|e| Error::Unexpected(e.into()))?;

            let nuc = row.nuclide;
            let decay_mode = row.decay_mode;

            let mut progeny = vec![];

            macro_rules! append_progeny {
                ($d:ident, $br:ident) => {
                    if row.$d.is_some() {
                        let d_nuc = row.$d.unwrap();
                        let mode = match_decay_mode(&nuc, &d_nuc, decay_mode)?;

                        progeny.push(Progeny {
                            mode,
                            branch_rate: row.$br.unwrap(),
                            nuclide: d_nuc,
                        })
                    }
                };
            }

            append_progeny!(d1, d1_branch);
            append_progeny!(d2, d2_branch);
            append_progeny!(d3, d3_branch);
            append_progeny!(d4, d4_branch);

            attribute.insert(
                nuc,
                Attribute {
                    half_life: row.half_life,
                    decay_mode,
                    progeny,
                    alpha_energy: row.alpha_energy,
                    electron_energy: row.electron_energy,
                    photon_energy: row.photon_energy,
                    n_photon_le_10kev_per_nt: row.n_photon_le_10kev_per_nt,
                    n_photon_gt_10kev_per_nt: row.n_photon_gt_10kev_per_nt,
                    n_beta_per_nt: row.n_beta_per_nt,
                    n_mono_electron_per_nt: row.n_mono_electron_per_nt,
                    n_alpha_per_nt: row.n_alpha_per_nt,
                    amu: row.amu,
                    air_kerma_const: row.air_kerma_const,
                    air_kerma_coef: row.air_kerma_coef,
                },
            );
        }

        macro_rules! read_spectrum {
            ($mod:ident, $type:ident, $file:expr, $range:expr) => {
                let mut spectrum_file: NuclideSpectrum<$mod::$type> =
                    NuclideSpectrum::new(path.as_ref().join($file), $range)?;
                for (n, s) in spectrum_file.0.drain() {
                    spectrum.insert(n, s.into_iter().map(std::convert::Into::into).collect());
                }
            };
        }

        read_spectrum!(rad, RadSpectrum, "ICRP-07.RAD", 20..29);
        read_spectrum!(bet, BetSpectrum, "ICRP-07.BET", 7..17);
        read_spectrum!(ack, AckSpectrum, "ICRP-07.ACK", 24..32);
        read_spectrum!(nsf, NsfSpectrum, "ICRP-07.NSF", 20..29);

        Ok(Self {
            attribute,
            spectrum,
        })
    }
}

fn match_decay_mode(
    parent: &Nuclide,
    daughter: &Nuclide,
    decay_mode: DecayMode,
) -> Result<DecayMode, Error> {
    let z = parent.symbol as u64;

    let d_z = daughter.symbol as u64;
    let mut mode = DecayMode::empty();

    if daughter.symbol == Symbol::SF {
        mode |= DecayMode::SPONTANEOUS_FISSION & decay_mode;
    } else {
        let a = parent.mass_number.unwrap();
        let d_a = daughter.mass_number.unwrap();

        if z == d_z && a == d_a {
            mode |= DecayMode::ISOMETRIC_TRANSITION & decay_mode;
        } else if z == d_z + 2 && a == d_a + 4 {
            mode |= DecayMode::ALPHA & decay_mode;
        } else if z + 1 == d_z && a == d_a {
            mode |= DecayMode::BETA_MINUS & decay_mode;
        } else if z == d_z + 1 && a == d_a {
            mode |= (DecayMode::BETA_PLUS | DecayMode::ELECTRON_CAPTURE) & decay_mode;
        }
    }

    if mode.is_empty() {
        Err(Error::Unexpected(anyhow::anyhow!(
            "unexpected decay mode: {} -> {}",
            parent,
            daughter
        )))
    } else {
        Ok(mode)
    }
}
