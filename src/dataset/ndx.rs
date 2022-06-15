use fixed_width::{FieldSet, FixedWidth};
use flagset::FlagSet;
use serde::Deserialize;

use super::reader;
use crate::error::Error;
use crate::nuclide::{decay_mode, DecayMode, HalfLife, Nuclide, Progeny, Symbol};

#[derive(Debug, Deserialize)]
pub(crate) struct NdxEntry {
    pub nuclide: Nuclide,
    pub half_life: HalfLife,
    #[serde(with = "decay_mode")]
    pub decay_mode: FlagSet<DecayMode>,
    pub progeny: Vec<Option<(Nuclide, f64)>>,
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
    fn fields() -> FieldSet {
        reader::fields_from_fortran_format(
            "(a7,a10,a8,28x,4(a7,6x,e11.0,1x),f7.0,2f8.0,3i4,i5,i4,e11.0,e10.0,e9.0)",
            0,
        )
        .unwrap()
        .0
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "NdxEntry")]
pub struct Attribute {
    pub half_life: HalfLife,
    pub decay_mode: FlagSet<DecayMode>,
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

impl FixedWidth for Attribute {
    fn fields() -> fixed_width::FieldSet {
        NdxEntry::fields()
    }
}

impl From<NdxEntry> for Attribute {
    fn from(entry: NdxEntry) -> Attribute {
        let mut progeny = vec![];

        for daughter in entry.progeny {
            match daughter {
                Some((nuclide, branch_rate)) => {
                    let mode =
                        check_decay_mode(&entry.nuclide, &nuclide, entry.decay_mode).unwrap();

                    progeny.push(Progeny {
                        decay_mode: mode,
                        branch_rate,
                        nuclide,
                    })
                }
                None => {}
            }
        }

        Attribute {
            half_life: entry.half_life,
            decay_mode: entry.decay_mode,
            progeny,
            alpha_energy: entry.alpha_energy,
            electron_energy: entry.electron_energy,
            photon_energy: entry.photon_energy,
            n_photon_le_10kev_per_nt: entry.n_photon_le_10kev_per_nt,
            n_photon_gt_10kev_per_nt: entry.n_photon_gt_10kev_per_nt,
            n_beta_per_nt: entry.n_beta_per_nt,
            n_mono_electron_per_nt: entry.n_mono_electron_per_nt,
            n_alpha_per_nt: entry.n_alpha_per_nt,
            amu: entry.amu,
            air_kerma_const: entry.air_kerma_const,
            air_kerma_coef: entry.air_kerma_coef,
        }
    }
}

fn check_decay_mode(
    parent: &Nuclide,
    daughter: &Nuclide,
    decay_mode: FlagSet<DecayMode>,
) -> Result<FlagSet<DecayMode>, Error> {
    let z = parent.symbol as u64;

    let d_z = daughter.symbol as u64;
    let mut mode = FlagSet::default();

    if daughter.symbol == Symbol::SF {
        mode |= DecayMode::SpontaneousFission & decay_mode;
    } else {
        let a = parent.mass_number.unwrap();
        let d_a = daughter.mass_number.unwrap();

        if z == d_z && a == d_a {
            mode |= DecayMode::IsometricTransition & decay_mode;
        } else if z == d_z + 2 && a == d_a + 4 {
            mode |= DecayMode::Alpha & decay_mode;
        } else if z + 1 == d_z && a == d_a {
            mode |= DecayMode::BetaMinus & decay_mode;
        } else if z == d_z + 1 && a == d_a {
            mode |= (DecayMode::BetaPlus | DecayMode::ElectronCapture) & decay_mode;
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

#[cfg(test)]
mod test {
    use super::{Attribute, NdxEntry};
    use crate::nuclide::Nuclide;
    use std::str::FromStr;

    #[test]
    fn test_nuclides_in_ndx_entry() {
        let data = "Ac-226    29.37h B-ECA      1944      1      0     0 Th-226   1108 8.3000E-01 Ra-226    822 1.7000E-01 Fr-222    361 6.0000E-05             0        0.0 0.0003 0.29143 0.13271  14 140   5   99   1 226.026097 1.048E-171.048E-17
";
        let entry: NdxEntry = fixed_width::from_str(data).unwrap();
        let attr: Attribute = fixed_width::from_str(data).unwrap();

        let parent = Nuclide::from_str("Ac-226").unwrap();
        assert_eq!(entry.nuclide, parent);

        let daughter1 = Nuclide::from_str("Th-226").unwrap();
        assert_eq!(entry.progeny[0].unwrap().0, daughter1);
        assert_eq!(attr.progeny[0].nuclide, daughter1);

        let daughter2 = Nuclide::from_str("Ra-226").unwrap();
        assert_eq!(entry.progeny[1].unwrap().0, daughter2);
        assert_eq!(attr.progeny[1].nuclide, daughter2);

        let daughter3 = Nuclide::from_str("Fr-222").unwrap();
        assert_eq!(entry.progeny[2].unwrap().0, daughter3);
        assert_eq!(attr.progeny[2].nuclide, daughter3);
    }
}
