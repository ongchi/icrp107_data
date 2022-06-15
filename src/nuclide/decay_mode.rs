use flagset::{flags, FlagSet};
use serde::{de::Visitor, Deserialize};

use crate::regex;

flags! {
    #[derive(Deserialize)]
    pub enum DecayMode: u8 {
        #[serde(rename = "A")]
        Alpha,
        #[serde(rename = "B-")]
        BetaMinus,
        #[serde(rename = "B+")]
        BetaPlus,
        #[serde(rename = "EC")]
        ElectronCapture,
        #[serde(rename = "IT")]
        IsometricTransition,
        #[serde(rename = "SF")]
        SpontaneousFission,
    }
}

impl std::fmt::Display for DecayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Alpha => "⍺",
                Self::BetaMinus => "β-",
                Self::BetaPlus => "β+",
                Self::ElectronCapture => "EC",
                Self::IsometricTransition => "IT",
                Self::SpontaneousFission => "SF",
            }
        )
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<FlagSet<DecayMode>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct DecayModeVisitor;

    impl<'de> Visitor<'de> for DecayModeVisitor {
        type Value = FlagSet<DecayMode>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A|B-|B+|EC|IT|SF")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let re = regex!(r"A|B\-|B\+|EC|IT|SF");

            let mut dm = FlagSet::default();
            for captures in re.captures_iter(v) {
                for capture in captures.iter() {
                    let mode: DecayMode = serde_plain::from_str(capture.unwrap().as_str())
                        .map_err(serde::de::Error::custom)?;
                    dm |= mode;
                }
            }

            Ok(dm)
        }
    }

    deserializer.deserialize_str(DecayModeVisitor)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_decay_mode() {
        let de = serde_plain::Deserializer::new("A ECB-");
        let mode = deserialize(de).unwrap();

        assert_eq!(
            mode,
            DecayMode::Alpha | DecayMode::ElectronCapture | DecayMode::BetaMinus
        );
    }
}
