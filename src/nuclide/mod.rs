pub mod decay_mode;
pub mod half_life;

pub use decay_mode::{DecayMode, DecayModePrimitive};
pub use half_life::HalfLife;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use crate::error::Error;
use crate::regex;

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum Symbol {
    H = 1, He, Li, Be, B, C, N, O, F, Ne, Na, Mg, Al, Si, P, S, Cl, Ar, K, Ca,
    Sc, Ti, V, Cr, Mn, Fe, Co, Ni, Cu, Zn, Ga, Ge, As, Se, Br, Kr, Rb, Sr, Y,
    Zr, Nb, Mo, Tc, Ru, Rh, Pd, Ag, Cd, In, Sn, Sb, Te, I, Xe, Cs, Ba, La, Ce,
    Pr, Nd, Pm, Sm, Eu, Gd, Tb, Dy, Ho, Er, Tm, Yb, Lu, Hf, Ta, W, Re, Os, Ir,
    Pt, Au, Hg, Tl, Pb, Bi, Po, At, Rn, Fr, Ra, Ac, Th, Pa, U, Np, Pu, Am, Cm,
    Bk, Cf, Es, Fm, Md, No, Lr, Rf, Db, Sg, Bh, Hs, Mt, Ds, Rg, Cn, Nh, Fl, Mc,
    Lv, Ts, Og,
}

serde_plain::derive_fromstr_from_deserialize!(Symbol, |e| -> Error {
    Error::InvalidSymbol(e.to_string())
});
serde_plain::derive_display_from_serialize!(Symbol);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum MetastableState {
    #[serde(rename = "m")]
    M = 1,
    #[serde(rename = "n")]
    N,
}

serde_plain::derive_fromstr_from_deserialize!(MetastableState, |e| -> Error {
    Error::InvalidState(e.to_string())
});
serde_plain::derive_display_from_serialize!(MetastableState);

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, DeserializeFromStr)]
pub struct Nuclide(
    /// Canonical id (zzzaaammmm)
    u32,
);

impl Nuclide {
    pub fn z(&self) -> u32 {
        self.0 / 10_000_000
    }

    pub fn a(&self) -> u32 {
        (self.0 / 10_000) % 1_000
    }

    pub fn symbol(&self) -> Symbol {
        num_traits::FromPrimitive::from_u8(self.z() as u8).unwrap()
    }

    pub fn state(&self) -> Option<MetastableState> {
        match self.0 % 10 {
            0 => None,
            state @ (1 | 2) => Some(num_traits::FromPrimitive::from_u8(state as u8)).unwrap(),
            _ => panic!("Invalid metastable state"),
        }
    }
}

impl std::fmt::Display for Nuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}{}",
            self.symbol(),
            self.a(),
            self.state().map_or("".to_string(), |m| m.to_string())
        )?;

        Ok(())
    }
}

impl FromStr for Nuclide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<symbol>\w+)((?:-)(?P<mass>\d+)(?P<state>\w)?)?");

        match re.captures(s) {
            Some(captures) => {
                let mut nuclide_id = 0;

                nuclide_id += match captures.name("symbol") {
                    Some(s) => s.as_str().parse::<Symbol>()?.to_u32().unwrap(),
                    None => 0,
                } * 10_000_000;

                nuclide_id += match captures.name("mass") {
                    Some(m) => {
                        let m = m.as_str();
                        m.parse()
                            .map_err(|_| Error::InvalidInteger(m.to_string()))?
                    }
                    None => 0,
                } * 10_000;

                nuclide_id += match captures.name("state") {
                    Some(s) => s.as_str().parse::<MetastableState>()?.to_u32().unwrap(),
                    None => 0,
                };

                Ok(Nuclide(nuclide_id))
            }
            None => Err(Error::InvalidNuclide(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DeserializeFromStr)]
pub enum MaybeNuclide {
    Nuclide(Nuclide),
    SF,
}

impl Display for MaybeNuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nuclide(n) => write!(f, "{}", n)?,
            Self::SF => write!(f, "various")?,
        }

        Ok(())
    }
}

impl FromStr for MaybeNuclide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "SF" {
            Ok(MaybeNuclide::SF)
        } else {
            Ok(MaybeNuclide::Nuclide(s.parse()?))
        }
    }
}

impl From<Nuclide> for MaybeNuclide {
    fn from(nuclide: Nuclide) -> Self {
        Self::Nuclide(nuclide)
    }
}

#[derive(Debug, Clone)]
pub struct Progeny {
    pub nuclide: MaybeNuclide,
    pub branch_rate: f64,
    pub decay_mode: DecayMode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nuclide_from_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(i131.0, 531310000);

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(tc99m.0, 430990001);
    }

    #[test]
    fn nuclide_to_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(&i131.to_string(), "I-131");

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(&tc99m.to_string(), "Tc-99m");
    }
}
