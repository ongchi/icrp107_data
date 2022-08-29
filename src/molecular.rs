use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use chumsky::prelude::*;

use crate::nuclide::Symbol;

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
