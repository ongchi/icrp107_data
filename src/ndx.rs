use fixed_width_derive::FixedWidth;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::nuclide::{DecayMode, HalfLife, Nuclide};
use super::spectrum::{NuclideSpectrum, Spectrum};
use super::{ack, bet, nsf, rad};
use super::{FileReader, ParseError};

#[derive(Debug, FixedWidth, Deserialize)]
struct NdxEntry {
    #[fixed_width(range = "0..7")]
    pub nuclide: Nuclide,
    #[fixed_width(range = "7..17")]
    pub half_life: HalfLife,
    #[fixed_width(range = "17..25")]
    pub decay_mode: DecayMode,
    #[fixed_width(range = "53..60")]
    pub d1: Option<Nuclide>,
    #[fixed_width(range = "66..77")]
    pub d1_branch: Option<f64>,
    #[fixed_width(range = "78..85")]
    pub d2: Option<Nuclide>,
    #[fixed_width(range = "91..102")]
    pub d2_branch: Option<f64>,
    #[fixed_width(range = "103..110")]
    pub d3: Option<Nuclide>,
    #[fixed_width(range = "116..127")]
    pub d3_branch: Option<f64>,
    #[fixed_width(range = "128..135")]
    pub d4: Option<Nuclide>,
    #[fixed_width(range = "141..152")]
    pub d4_branch: Option<f64>,
    #[fixed_width(range = "152..159")]
    pub alpha_energy: f64,
    #[fixed_width(range = "159..167")]
    pub electron_energy: f64,
    #[fixed_width(range = "167..175")]
    pub photon_energy: f64,
    #[fixed_width(range = "175..179")]
    pub n_photon_le_10kev_per_nt: u64,
    #[fixed_width(range = "179..183")]
    pub n_photon_gt_10kev_per_nt: u64,
    #[fixed_width(range = "183..187")]
    pub n_beta_per_nt: u64,
    #[fixed_width(range = "187..192")]
    pub n_mono_electron_per_nt: u64,
    #[fixed_width(range = "192..196")]
    pub n_alpha_per_nt: u64,
    #[fixed_width(range = "196..207")]
    pub amu: f64,
    #[fixed_width(range = "207..217")]
    pub air_kerma_const: f64,
    #[fixed_width(range = "217..226")]
    pub air_kerma_coef: f64,
}

#[derive(Debug)]
pub struct Progeny {
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
    pub spectrum: Vec<Spectrum>,
}

#[derive(Debug)]
pub struct NuclideData(pub HashMap<Nuclide, Attribute>);

impl NuclideData {
    pub fn open<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let mut ndx = FileReader::new(&path.as_ref().join("ICRP-07.NDX")).skip_lines(1);
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while ndx.read_buf(&mut buf)? != 0 {
            let row: NdxEntry =
                fixed_width::from_str(&buf).map_err(|e| ParseError::UnexpectedError(e.into()))?;

            let mut progeny = vec![];

            macro_rules! append_progeny {
                ($d:ident, $br:ident) => {
                    if row.$d.is_some() {
                        progeny.push(Progeny {
                            branch_rate: row.$br.unwrap(),
                            nuclide: row.$d.unwrap(),
                        })
                    }
                };
            }

            append_progeny!(d1, d1_branch);
            append_progeny!(d2, d2_branch);
            append_progeny!(d3, d3_branch);
            append_progeny!(d4, d4_branch);

            inner.insert(
                row.nuclide,
                Attribute {
                    half_life: row.half_life,
                    decay_mode: row.decay_mode,
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
                    spectrum: Vec::new(),
                },
            );
        }

        macro_rules! read_spectrum {
            ($mod:ident, $type:ident, $file:expr, $range:expr) => {
                let mut spectrum_file: NuclideSpectrum<$mod::Spectrum> =
                    NuclideSpectrum::new(path.as_ref().join($file), $range)?;
                for (nuclide, spectrum) in spectrum_file.0.drain() {
                    if let Some(attr) = inner.get_mut(&nuclide) {
                        attr.spectrum
                            .extend(spectrum.into_iter().map(Spectrum::$type))
                    };
                }
            };
        }

        read_spectrum!(rad, Radiation, "ICRP-07.RAD", 20..29);
        read_spectrum!(bet, Beta, "ICRP-07.BET", 7..17);
        read_spectrum!(ack, AugerCKElectron, "ICRP-07.ACK", 24..32);
        read_spectrum!(nsf, Neutron, "ICRP-07.NSF", 20..29);

        Ok(Self(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::NuclideData;

    #[test]
    fn foobar() {
        let data = NuclideData::open("./").unwrap();

        assert_eq!(data.0.len(), 1252);
    }
}
