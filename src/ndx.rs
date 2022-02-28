use fixed_width::{Field, FixedWidth};
use flagset::FlagSet;
use serde::Deserialize;

use crate::derive_fixed_width_from_fortran_format;
use crate::error::Error;
use crate::nuclide::de_decay_mode;
use crate::{DecayMode, HalfLife, Nuclide, Symbol};

#[derive(Debug, Deserialize)]
pub(crate) struct NdxEntry {
    pub nuclide: Nuclide,
    pub half_life: HalfLife,
    #[serde(deserialize_with = "de_decay_mode")]
    pub decay_mode: FlagSet<DecayMode>,
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
    pub mode: FlagSet<DecayMode>,
    pub branch_rate: f64,
    pub nuclide: Nuclide,
}

#[derive(Debug)]
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

impl From<NdxEntry> for Attribute {
    fn from(entry: NdxEntry) -> Attribute {
        let mut progeny = vec![];

        macro_rules! append_progeny {
            ($d:ident, $br:ident) => {
                if entry.$d.is_some() {
                    let d_nuc = entry.$d.unwrap();
                    let mode = check_decay_mode(&entry.nuclide, &d_nuc, entry.decay_mode).unwrap();

                    progeny.push(Progeny {
                        mode,
                        branch_rate: entry.$br.unwrap(),
                        nuclide: d_nuc,
                    })
                }
            };
        }

        append_progeny!(d1, d1_branch);
        append_progeny!(d2, d2_branch);
        append_progeny!(d3, d3_branch);
        append_progeny!(d4, d4_branch);

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
