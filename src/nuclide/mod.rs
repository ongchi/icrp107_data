pub mod half_life;

use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::hash::Hash;

use crate::error::Error;
use crate::regex;

#[rustfmt::skip]
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Symbol {
    SF, H , He, Li, Be, B, C, N, O, F, Ne, Na, Mg, Al, Si, P, S, Cl, Ar, K, Ca,
    Sc, Ti, V, Cr, Mn, Fe, Co, Ni, Cu, Zn, Ga, Ge, As, Se, Br, Kr, Rb, Sr, Y,
    Zr, Nb, Mo, Tc, Ru, Rh, Pd, Ag, Cd, In, Sn, Sb, Te, I, Xe, Cs, Ba, La, Ce,
    Pr, Nd, Pm, Sm, Eu, Gd, Tb, Dy, Ho, Er, Tm, Yb, Lu, Hf, Ta, W, Re, Os, Ir,
    Pt, Au, Hg, Tl, Pb, Bi, Po, At, Rn, Fr, Ra, Ac, Th, Pa, U, Np, Pu, Am, Cm,
    Bk, Cf, Es, Fm, Md, No, Lr, Rf, Db, Sg, Bh, Hs, Mt, Ds, Rg, Cn, Nh, Fl, Mc,
    Lv, Ts, Og,
}

serde_plain::derive_fromstr_from_deserialize!(Symbol);
serde_plain::derive_display_from_serialize!(Symbol);

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetastableState {
    #[serde(rename = "m")]
    M,
    #[serde(rename = "n")]
    N,
}

serde_plain::derive_fromstr_from_deserialize!(MetastableState);
serde_plain::derive_display_from_serialize!(MetastableState);

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, DeserializeFromStr)]
pub struct Nuclide {
    pub symbol: Symbol,
    pub mass_number: Option<u64>,
    pub meta: Option<MetastableState>,
}

impl std::fmt::Display for Nuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.symbol.to_string());
        if self.mass_number.is_some() {
            s.push_str(&format!("-{}", &self.mass_number.unwrap()));
            if self.meta.is_some() {
                s.push_str(&format!("{}", &self.meta.unwrap()));
            }
        }
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Nuclide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<symbol>\w+)((?:-)(?P<mass>\d+)(?P<meta>\w)?)?");

        let captures = re
            .captures(s)
            .ok_or_else(|| Error::InvalidNuclide(s.to_string()))?;

        let symbol = captures.name("symbol").unwrap().as_str().parse().unwrap();
        let mass_number = captures.name("mass").map(|a| a.as_str().parse().unwrap());
        let meta = captures.name("meta").map(|m| m.as_str().parse().unwrap());

        Ok(Self {
            symbol,
            mass_number,
            meta,
        })
    }
}

bitflags::bitflags! {
    #[derive(DeserializeFromStr)]
    pub struct DecayMode: u8 {
        const ALPHA = 0x01;
        const BETA_MINUS = 0x02;
        const BETA_PLUS = 0x04;
        const ELECTRON_CAPTURE = 0x08;
        const ISOMETRIC_TRANSITION = 0x10;
        const SPONTANEOUS_FISSION = 0x20;
    }
}

impl std::str::FromStr for DecayMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"A|B\-|B\+|EC|IT|SF");

        let mut dm: DecayMode = DecayMode::empty();
        for captures in re.captures_iter(s) {
            for capture in captures.iter() {
                match capture.unwrap().as_str() {
                    "A" => dm |= DecayMode::ALPHA,
                    "B-" => dm |= DecayMode::BETA_MINUS,
                    "B+" => dm |= DecayMode::BETA_PLUS,
                    "EC" => dm |= DecayMode::ELECTRON_CAPTURE,
                    "IT" => dm |= DecayMode::ISOMETRIC_TRANSITION,
                    "SF" => dm |= DecayMode::SPONTANEOUS_FISSION,
                    dm => return Err(Error::InvalidDecayMode(dm.to_string())),
                }
            }
        }

        Ok(dm)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nuclide_from_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(i131.symbol, Symbol::I);
        assert_eq!(i131.mass_number, Some(131));
        assert_eq!(i131.meta, None);

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(tc99m.symbol, Symbol::Tc);
        assert_eq!(tc99m.mass_number, Some(99));
        assert_eq!(tc99m.meta, Some(MetastableState::M));
    }

    #[test]
    fn nuclide_to_string() {
        let i131: Nuclide = "I-131".parse().unwrap();
        assert_eq!(&i131.to_string(), "I-131");

        let tc99m: Nuclide = "Tc-99m".parse().unwrap();
        assert_eq!(&tc99m.to_string(), "Tc-99m");
    }
}
