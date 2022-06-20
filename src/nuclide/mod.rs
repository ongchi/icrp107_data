pub mod decay_mode;
pub mod half_life;

pub use decay_mode::{DecayMode, DecayModePrimitive};
pub use half_life::HalfLife;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use crate::error::Error;
use crate::regex;

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive)]
pub enum Symbol {
    H = 1, He, Li, Be, B, C, N, O, F, Ne, Na, Mg, Al, Si, P, S, Cl, Ar, K, Ca,
    Sc, Ti, V, Cr, Mn, Fe, Co, Ni, Cu, Zn, Ga, Ge, As, Se, Br, Kr, Rb, Sr, Y,
    Zr, Nb, Mo, Tc, Ru, Rh, Pd, Ag, Cd, In, Sn, Sb, Te, I, Xe, Cs, Ba, La, Ce,
    Pr, Nd, Pm, Sm, Eu, Gd, Tb, Dy, Ho, Er, Tm, Yb, Lu, Hf, Ta, W, Re, Os, Ir,
    Pt, Au, Hg, Tl, Pb, Bi, Po, At, Rn, Fr, Ra, Ac, Th, Pa, U, Np, Pu, Am, Cm,
    Bk, Cf, Es, Fm, Md, No, Lr, Rf, Db, Sg, Bh, Hs, Mt, Ds, Rg, Cn, Nh, Fl, Mc,
    Lv, Ts, Og,
}

impl Symbol {
    pub fn from_z(z: u8) -> Option<Self> {
        FromPrimitive::from_u8(z)
    }
}

serde_plain::derive_fromstr_from_deserialize!(Symbol, |e| -> Error {
    Error::InvalidSymbol(e.to_string())
});
serde_plain::derive_display_from_serialize!(Symbol);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, DeserializeFromStr)]
pub enum Nuclide {
    /// Nuclide with canonical id
    WithId(u32),
    /// Spontaneous fission products
    FissionProducts,
}

impl Nuclide {
    pub fn id(&self) -> Option<u32> {
        match self {
            Self::WithId(id) => Some(*id),
            Self::FissionProducts => None,
        }
    }

    pub fn z(&self) -> Option<u32> {
        self.id().map(|id| id / 10_000_000)
    }

    pub fn a(&self) -> Option<u32> {
        self.id().map(|id| (id / 10_000) % 1_000)
    }

    pub fn state(&self) -> Option<MetastableState> {
        match self.id() {
            Some(id) => FromPrimitive::from_u8((id % 10) as u8),
            None => None,
        }
    }
}

impl Display for Nuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WithId(_) => {
                write!(
                    f,
                    "{}-{}{}",
                    Symbol::from_z(self.z().unwrap() as u8).unwrap(),
                    self.a().unwrap(),
                    self.state().map_or("".to_string(), |m| m.to_string())
                )?;
            }
            Self::FissionProducts => write!(f, "various")?,
        }

        Ok(())
    }
}

impl FromStr for Nuclide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<symbol>\w+)((?:-)(?P<mass>\d+)(?P<state>\w)?)?");

        match re.captures(s) {
            Some(captures) => {
                let symbol_str = captures
                    .name("symbol")
                    .map(|m| m.as_str())
                    .ok_or(Error::InvalidNuclide(s.to_string()))?;

                if symbol_str == "SF" {
                    Ok(Self::FissionProducts)
                } else {
                    let mut id = 0;

                    id += (symbol_str.parse::<Symbol>()? as u32) * 10_000_000;

                    id += match captures.name("mass") {
                        Some(m) => {
                            let m = m.as_str();
                            m.parse()
                                .map_err(|_| Error::InvalidInteger(m.to_string()))?
                        }
                        None => 0,
                    } * 10_000;

                    id += match captures.name("state") {
                        Some(s) => s.as_str().parse::<MetastableState>()? as u32,
                        None => 0,
                    };

                    Ok(Self::WithId(id))
                }
            }
            None => Err(Error::InvalidNuclide(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Progeny {
    pub nuclide: Nuclide,
    pub branch_rate: f64,
    pub decay_mode: DecayMode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nuclide_from_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(i131.id().unwrap(), 531310000);

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(tc99m.id().unwrap(), 430990001);
    }

    #[test]
    fn nuclide_to_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(&i131.to_string(), "I-131");

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(&tc99m.to_string(), "Tc-99m");
    }
}
