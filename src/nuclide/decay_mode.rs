use flagset::{flags, FlagSet};
use serde::{de::Visitor, Deserialize};

use crate::regex;

pub type DecayMode = FlagSet<DecayModePrimitive>;

flags! {
    #[derive(Deserialize)]
    pub enum DecayModePrimitive: u8 {
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

impl std::fmt::Display for DecayModePrimitive {
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

pub fn deserialize<'de, D>(deserializer: D) -> Result<DecayMode, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct DecayModeVisitor;

    impl<'de> Visitor<'de> for DecayModeVisitor {
        type Value = DecayMode;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A|B-|B+|EC|IT|SF")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let re = regex!(r"A|B\-|B\+|EC|IT|SF");

            let mut mode = DecayMode::default();
            for captures in re.captures_iter(v) {
                for capture in captures.iter() {
                    let m: DecayModePrimitive = serde_plain::from_str(capture.unwrap().as_str())
                        .map_err(serde::de::Error::custom)?;
                    mode |= m;
                }
            }

            Ok(mode)
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
            DecayModePrimitive::Alpha
                | DecayModePrimitive::ElectronCapture
                | DecayModePrimitive::BetaMinus
        );
    }
}
