use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use chumsky::prelude::{end, filter, just, recursive, Parser, Simple};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::primitive::attr::AtomicMass;

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, FromPrimitive)]
pub enum Symbol {
    H = 1, He, Li, Be, B, C, N, O, F, Ne, Na, Mg, Al, Si, P, S, Cl, Ar, K, Ca,
    Sc, Ti, V, Cr, Mn, Fe, Co, Ni, Cu, Zn, Ga, Ge, As, Se, Br, Kr, Rb, Sr, Y,
    Zr, Nb, Mo, Tc, Ru, Rh, Pd, Ag, Cd, In, Sn, Sb, Te, I, Xe, Cs, Ba, La, Ce,
    Pr, Nd, Pm, Sm, Eu, Gd, Tb, Dy, Ho, Er, Tm, Yb, Lu, Hf, Ta, W, Re, Os, Ir,
    Pt, Au, Hg, Tl, Pb, Bi, Po, At, Rn, Fr, Ra, Ac, Th, Pa, U, Np, Pu, Am, Cm,
    Bk, Cf, Es, Fm, Md, No, Lr, Rf, Db, Sg, Bh, Hs, Mt, Ds, Rg, Cn, Nh, Fl, Mc,
    Lv, Ts, Og,
}

impl TryFrom<u8> for Symbol {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(Error::InvalidAtomicNumber(value))
    }
}

serde_plain::derive_fromstr_from_deserialize!(Symbol, |e| -> Error {
    Error::InvalidSymbol(e.to_string())
});
serde_plain::derive_display_from_serialize!(Symbol);

#[derive(Debug)]
pub enum Molecular {
    Element(Symbol, u32),
    Compound(Vec<Molecular>, u32),
}

impl Display for Molecular {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Element(symbol, n) => {
                write!(f, "{}", symbol)?;
                if n != &1 {
                    write!(f, "{}", n)?;
                }
            }
            Self::Compound(g, mul) => {
                if mul != &1 {
                    write!(f, "(")?;
                }
                for el in g {
                    el.fmt(f)?;
                }
                if mul != &1 {
                    write!(f, "){}", mul)?;
                }
            }
        };
        Ok(())
    }
}

impl FromStr for Molecular {
    type Err = Vec<Simple<char>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        molecular_parser().parse(s)
    }
}

impl Molecular {
    pub fn composition(&self) -> BTreeMap<Symbol, u32> {
        let mut comp = BTreeMap::new();

        match self {
            Self::Element(symbol, n) => *comp.entry(*symbol).or_insert(0) += n,
            Self::Compound(g, mul) => {
                for el in g {
                    for (symbol, n) in el.composition().iter() {
                        *comp.entry(*symbol).or_insert(0) += mul * n;
                    }
                }
            }
        }

        comp
    }
}

fn molecular_parser() -> impl Parser<char, Molecular, Error = Simple<char>> {
    let number = filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .map(|s| s.into_iter().collect::<String>().parse().unwrap_or(1));

    let symbol = filter(|c: &char| c.is_ascii_uppercase())
        .chain(filter(|c: &char| c.is_ascii_lowercase()).repeated())
        .try_map(|chs, span| {
            chs.into_iter()
                .collect::<String>()
                .parse::<Symbol>()
                .map_err(|e| Simple::custom(span, format!("{}", e)))
        });

    let element = symbol.then(number).map(|(s, n)| Molecular::Element(s, n));

    let compound = recursive(|expr| {
        element
            .or(expr
                .delimited_by(just('('), just(')'))
                .then(number)
                .map(|(mole, n)| Molecular::Compound(mole, n)))
            .repeated()
            .at_least(1)
    });

    compound.then_ignore(end()).map(|mole| {
        if mole.len() == 1 {
            mole.into_iter().next().unwrap()
        } else {
            Molecular::Compound(mole, 1)
        }
    })
}

pub struct MaterialBuilder<'a> {
    data: &'a dyn AtomicMass,
    composition: BTreeMap<Symbol, f64>,
    weight_fraction: BTreeMap<Symbol, f64>,
    density: Option<f64>,
    weight: Option<f64>,
}

impl<'a> MaterialBuilder<'a> {
    pub fn new(data: &'a dyn AtomicMass) -> Self {
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
            let mass = n * self.data.atomic_mass(symbol)?;
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
            let n = f / self.data.atomic_mass(symbol)?;
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn molecular_formula_parser() {
        let ether: Molecular = "(C2H5)2O".parse().unwrap();
        let mut composition: BTreeMap<Symbol, u32> = BTreeMap::new();
        composition.insert(Symbol::H, 10);
        composition.insert(Symbol::C, 4);
        composition.insert(Symbol::O, 1);

        assert_eq!(format!("{}", ether), "(C2H5)2O");
        assert_eq!(ether.composition(), composition);
    }
}
