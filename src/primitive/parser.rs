use chumsky::prelude::{filter, just, recursive, text, Parser, Simple};
use chumsky::text::TextParser;
use flagset::FlagSet;

use super::notation::{Compound, Symbol};
use super::nuclide::{DecayMode, HalfLife, MetastableState, Nuclide, TimeUnit};

pub fn symbol() -> impl Parser<char, Symbol, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_uppercase())
        .chain(filter(|c: &char| c.is_ascii_lowercase()).repeated())
        .try_map(|chs, span| {
            chs.into_iter()
                .collect::<String>()
                .parse::<Symbol>()
                .map_err(|e| Simple::custom(span, format!("{}", e)))
        })
}

pub fn nuclide() -> impl Parser<char, Nuclide, Error = Simple<char>> {
    let sf = text::keyword("SF").map(|_| Nuclide::FissionProducts);

    let symbol = symbol().map(|s| s as u32 * 10_000_000);

    let mass = text::int(10)
        .try_map(|s: String, span| {
            s.parse::<u32>()
                .map_err(|e| Simple::custom(span, format!("{}", e)))
        })
        .map(|m| m * 10_000);

    let meta = filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .map(|meta| {
            meta.into_iter()
                .collect::<String>()
                .parse::<MetastableState>()
                .map(|m| m as u32)
                .unwrap_or(0)
        });

    let with_name = sf.or(symbol
        .chain(just('-').repeated().map(|_| 0u32))
        .chain(mass)
        .chain(meta)
        .map(|v| Nuclide::WithId(v.into_iter().sum())));

    let with_id = text::int(10).try_map(|s: String, span| {
        s.parse::<u32>()
            .map_err(|e| Simple::custom(span, format!("{}", e)))
            .map(Nuclide::WithId)
    });

    with_name.or(with_id)
}

pub fn decaymode() -> impl Parser<char, DecayMode, Error = Simple<char>> {
    let a = just("A").or(just("⍺")).map(|_| DecayMode::Alpha).padded();
    let bm = just("B-")
        .or(just("β-"))
        .map(|_| DecayMode::BetaMinus)
        .padded();
    let bp = just("B+")
        .or(just("β+"))
        .map(|_| DecayMode::BetaPlus)
        .padded();
    let ec = just("EC").map(|_| DecayMode::ElectronCapture).padded();
    let it = just("IT").map(|_| DecayMode::IsometricTransition).padded();
    let sf = just("SF").map(|_| DecayMode::SpontaneousFission).padded();

    a.or(bm.or(bp.or(ec.or(it.or(sf)))))
}

pub fn decaymodeflags() -> impl Parser<char, FlagSet<DecayMode>, Error = Simple<char>> {
    decaymode()
        .repeated()
        .map(|modes| modes.into_iter().fold(FlagSet::default(), |a, b| a | b))
}

pub fn compound() -> impl Parser<char, Compound, Error = Simple<char>> {
    let number = filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .map(|s| s.into_iter().collect::<String>().parse().unwrap_or(1));

    let compound = recursive(|expr| {
        symbol()
            .then(number)
            .map(|(s, n)| Compound::Element(s, n))
            .or(expr
                .delimited_by(just('('), just(')'))
                .then(number)
                .map(|(mole, n)| Compound::Molecule(mole, n)))
            .repeated()
            .at_least(1)
    });

    compound.map(|mole| {
        if mole.len() == 1 {
            mole.into_iter().next().unwrap()
        } else {
            Compound::Molecule(mole, 1)
        }
    })
}

pub fn float() -> impl Parser<char, f64, Error = Simple<char>> {
    let ch = |c: char| just(c).map(|v| vec![v]);

    let sign = ch('+').or(ch('-')).or_not().map(|v| v.unwrap_or_default());
    let e = ch('e').or(ch('E'));

    let nums = filter(|c: &char| c.is_ascii_digit()).repeated();
    let signed_nums = sign.chain(nums.at_least(1)).map(|v: Vec<char>| v);
    let float = signed_nums
        .chain(
            ch('.')
                .chain(nums.or_not().map(|v| v.unwrap_or_default()))
                .or_not()
                .map(|v: Option<Vec<char>>| v.unwrap_or_default()),
        )
        .map(|v: Vec<char>| v);
    let scientific = float.chain(
        e.chain(signed_nums)
            .or_not()
            .map(|v: Option<Vec<char>>| v.unwrap_or_default()),
    );

    scientific.try_map(|chs: Vec<char>, span| {
        chs.into_iter()
            .collect::<String>()
            .parse::<f64>()
            .map_err(|e| Simple::custom(span, format!("{}", e)))
    })
}

pub fn halflife() -> impl Parser<char, HalfLife, Error = Simple<char>> {
    let us = just("us").map(|_| TimeUnit::MicroSecond);
    let ms = just("ms").map(|_| TimeUnit::MicroSecond);
    let s = just("s").map(|_| TimeUnit::Second);
    let m = just("m").map(|_| TimeUnit::Minute);
    let h = just("h").map(|_| TimeUnit::Hour);
    let d = just("d").map(|_| TimeUnit::Day);
    let y = just("y").map(|_| TimeUnit::Year);

    let unit = us.or(ms.or(s.or(m.or(h.or(d.or(y))))));

    float()
        .padded()
        .then(unit)
        .map(|(value, unit)| HalfLife { value, unit })
}

pub fn gi_absorption_factor() -> impl Parser<char, (f64, String), Error = Simple<char>> {
    let compound = filter(|c: &char| c.is_ascii_alphanumeric())
        .repeated()
        .map(|s| s.into_iter().collect::<String>());

    float().padded().then(compound)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_symbol() {
        let h = symbol().parse("H").unwrap();
        assert_eq!(h, Symbol::H);

        let he = symbol().parse("He").unwrap();
        assert_eq!(he, Symbol::He);
    }

    #[test]
    fn parse_nuclide() {
        let tc99 = nuclide().parse("Tc99").unwrap();
        assert_eq!(tc99, Nuclide::WithId(43_099_0000));

        let tc99m = nuclide().parse("Tc-99m").unwrap();
        assert_eq!(tc99m, Nuclide::WithId(43_099_0001));

        let tc99m_from_id = nuclide().parse("430990001").unwrap();
        assert_eq!(tc99m_from_id, Nuclide::WithId(43_099_0001));
    }

    #[test]
    fn parse_decaymodeflags() {
        // formatter.write_str("A|B-|B+|EC|IT|SF")
        let mode = decaymodeflags().parse("AB-B+ECITSF").unwrap();
        assert_eq!(
            mode,
            FlagSet::default()
                | DecayMode::Alpha
                | DecayMode::BetaMinus
                | DecayMode::BetaPlus
                | DecayMode::ElectronCapture
                | DecayMode::IsometricTransition
                | DecayMode::SpontaneousFission
        );

        let mode_with_padding = decaymodeflags().parse("A B- β+ ").unwrap();
        assert_eq!(
            mode_with_padding,
            FlagSet::default() | DecayMode::Alpha | DecayMode::BetaMinus | DecayMode::BetaPlus
        );
    }

    #[test]
    fn parse_compound() {
        let ether = compound().parse("(C2H5)2O").unwrap();
        assert_eq!(
            ether,
            Compound::Molecule(
                vec![
                    Compound::Molecule(
                        vec![
                            Compound::Element(Symbol::C, 2),
                            Compound::Element(Symbol::H, 5),
                        ],
                        2
                    ),
                    Compound::Element(Symbol::O, 1)
                ],
                1
            )
        );
    }

    #[test]
    fn parse_float() {
        let f1 = float().parse("1").unwrap();
        assert_eq!(f1, 1.0);

        let f2 = float().parse("2.").unwrap();
        assert_eq!(f2, 2.0);

        let f3 = float().parse("1.2").unwrap();
        assert_eq!(f3, 1.2);

        let f4 = float().parse("1.23e01").unwrap();
        assert_eq!(f4, 12.3);

        let f5 = float().parse("-1.234e-2").unwrap();
        assert_eq!(f5, -0.01234);
    }

    #[test]
    fn parse_halflife() {
        let h1 = halflife().parse("1.23e-2s").unwrap();
        assert_eq!(
            h1,
            HalfLife {
                value: 0.0123,
                unit: TimeUnit::Second
            }
        );

        let h2 = halflife().parse("321 h").unwrap();
        assert_eq!(
            h2,
            HalfLife {
                value: 321.,
                unit: TimeUnit::Hour
            }
        )
    }

    #[test]
    fn parse_gi_absorption_factor() {
        let f1_1 = gi_absorption_factor().parse("1OBT").unwrap();
        assert_eq!(f1_1, (1., "OBT".to_string()));

        let f1_2 = gi_absorption_factor().parse("1 CH4").unwrap();
        assert_eq!(f1_2, (1., "CH4".to_string()));
    }
}
