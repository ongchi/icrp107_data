use fixed_width::{Field, FixedWidth};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::nuclide::{DecayMode, HalfLife, Nuclide};
use super::spectrum::{NuclideSpectrum, Spectrum};
use super::{ack, bet, nsf, rad};
use super::{FileReader, ParseError};

#[derive(Debug, Deserialize)]
struct NdxEntry {
    pub nuclide: Nuclide,
    pub half_life: HalfLife,
    pub decay_mode: DecayMode,
    // pub rad_index: u64,
    // pub bet_index: u64,
    // pub ack_index: u64,
    // pub nsf_index: u64,
    pub d1: Option<Nuclide>,
    // pub d1_ndx_index: Option<u64>,
    pub d1_branch: Option<f64>,
    pub d2: Option<Nuclide>,
    // pub d2_ndx_index: Option<u64>,
    pub d2_branch: Option<f64>,
    pub d3: Option<Nuclide>,
    // pub d3_ndx_index: Option<u64>,
    pub d3_branch: Option<f64>,
    pub d4: Option<Nuclide>,
    // pub d4_ndx_index: Option<u64>,
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

impl FixedWidth for NdxEntry {
    fn fields() -> Vec<Field> {
        vec![
            Field::default().range(0..7),
            Field::default().range(7..17),
            Field::default().range(17..25),
            // Field::default().range(25..32),
            // Field::default().range(32..39),
            // Field::default().range(39..46),
            // Field::default().range(46..52),
            Field::default().range(53..60),
            // Field::default().range(60..66),
            Field::default().range(66..77),
            Field::default().range(78..85),
            // Field::default().range(85..91),
            Field::default().range(91..102),
            Field::default().range(103..110),
            // Field::default().range(110..116),
            Field::default().range(116..127),
            Field::default().range(128..135),
            // Field::default().range(135..141),
            Field::default().range(141..152),
            Field::default().range(152..159),
            Field::default().range(159..167),
            Field::default().range(167..175),
            Field::default().range(175..179),
            Field::default().range(179..183),
            Field::default().range(183..187),
            Field::default().range(187..192),
            Field::default().range(192..196),
            Field::default().range(196..207),
            Field::default().range(207..217),
            Field::default().range(217..226),
        ]
    }
}

#[derive(Debug)]
pub struct Daughter {
    pub branch_rate: f64,
    pub nuclide: Nuclide,
}

#[derive(Debug)]
pub struct Attribute {
    pub half_life: HalfLife,
    pub decay_mode: DecayMode,
    pub daughter: Vec<Daughter>,
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

            let mut daughter = vec![];
            if row.d1.is_some() {
                daughter.push(Daughter {
                    branch_rate: row.d1_branch.unwrap(),
                    nuclide: row.d1.unwrap(),
                })
            };
            if row.d2.is_some() {
                daughter.push(Daughter {
                    branch_rate: row.d2_branch.unwrap(),
                    nuclide: row.d2.unwrap(),
                })
            };
            if row.d3.is_some() {
                daughter.push(Daughter {
                    branch_rate: row.d3_branch.unwrap(),
                    nuclide: row.d3.unwrap(),
                })
            };
            if row.d4.is_some() {
                daughter.push(Daughter {
                    branch_rate: row.d4_branch.unwrap(),
                    nuclide: row.d4.unwrap(),
                })
            };

            inner.insert(
                row.nuclide,
                Attribute {
                    half_life: row.half_life,
                    decay_mode: row.decay_mode,
                    daughter,
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

        let mut rad: NuclideSpectrum<rad::Spectrum> =
            NuclideSpectrum::new::<_, rad::Entry>(path.as_ref().join("ICRP-07.RAD"))?;
        for (nuclide, spectrum) in rad.0.drain() {
            if let Some(attr) = inner.get_mut(&nuclide) {
                attr.spectrum
                    .extend(spectrum.into_iter().map(|s| Spectrum::Radiation(s)))
            };
        }

        let mut bet: NuclideSpectrum<bet::Spectrum> =
            NuclideSpectrum::new::<_, bet::Entry>(path.as_ref().join("ICRP-07.BET"))?;
        for (nuclide, spectrum) in bet.0.drain() {
            if let Some(attr) = inner.get_mut(&nuclide) {
                attr.spectrum
                    .extend(spectrum.into_iter().map(|s| Spectrum::Beta(s)))
            };
        }

        let mut ack: NuclideSpectrum<ack::Spectrum> =
            NuclideSpectrum::new::<_, ack::Entry>(path.as_ref().join("ICRP-07.ACK"))?;
        for (nuclide, spectrum) in ack.0.drain() {
            if let Some(attr) = inner.get_mut(&nuclide) {
                attr.spectrum
                    .extend(spectrum.into_iter().map(|s| Spectrum::AugerCkElectron(s)))
            };
        }

        let mut nsf: NuclideSpectrum<nsf::Spectrum> =
            NuclideSpectrum::new::<_, nsf::Entry>(path.as_ref().join("ICRP-07.NSF"))?;
        for (nuclide, spectrum) in nsf.0.drain() {
            if let Some(attr) = inner.get_mut(&nuclide) {
                attr.spectrum
                    .extend(spectrum.into_iter().map(|s| Spectrum::Neutron(s)))
            };
        }

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
