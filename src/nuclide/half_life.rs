use serde::Deserialize;
use serde_with::DeserializeFromStr;

use crate::error::Error;
use crate::regex;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum TimeUnit {
    #[serde(rename = "us")]
    MicroSecond,
    #[serde(rename = "ms")]
    MilliSecond,
    #[serde(rename = "s")]
    Second,
    #[serde(rename = "m")]
    Minute,
    #[serde(rename = "h")]
    Hour,
    #[serde(rename = "d")]
    Day,
    #[serde(rename = "y")]
    Year,
}

serde_plain::derive_fromstr_from_deserialize!(TimeUnit);

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MicroSecond => "μs",
                Self::MilliSecond => "ms",
                Self::Second => "s",
                Self::Minute => "m",
                Self::Hour => "h",
                Self::Day => "d",
                Self::Year => "y",
            }
        )
    }
}

impl TimeUnit {
    pub fn as_sec(&self) -> f64 {
        match self {
            Self::MicroSecond => 1e-6,
            Self::MilliSecond => 1e-3,
            Self::Second => 1.,
            Self::Minute => 60.,
            Self::Hour => 3_600.,
            Self::Day => 86_400.,
            Self::Year => 365.2422 * 86400.,
        }
    }
}

#[derive(Debug, DeserializeFromStr)]
pub struct HalfLife {
    pub value: f64,
    pub unit: TimeUnit,
}

impl HalfLife {
    pub fn as_sec(&self) -> f64 {
        self.value * self.unit.as_sec()
    }
}

impl std::str::FromStr for HalfLife {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex!(r"(?P<value>\d+\.?(?:\d+)?(?:[Ee][+-]?\d+)?)(?P<unit>(?:[um]?s)|m|h|d|y)");

        let captures = re
            .captures(s)
            .ok_or_else(|| Error::InvalidHalfLife(s.to_string()))?;

        let value = captures.name("value").unwrap().as_str().parse().unwrap();
        let unit = captures.name("unit").unwrap().as_str().parse().unwrap();

        Ok(Self { value, unit })
    }
}

impl std::fmt::Display for HalfLife {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.value, self.unit.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn isclose(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-8 + 1e-5 * b.abs()
    }

    #[test]
    fn halflife_from_string() {
        let t1: HalfLife = "1us".parse().unwrap();
        assert!(isclose(t1.value, 1.));
        assert_eq!(t1.unit, TimeUnit::MicroSecond);

        let t2: HalfLife = "2h".parse().unwrap();
        assert!(isclose(t2.value, 2.));
        assert_eq!(t2.unit, TimeUnit::Hour);

        let t3: HalfLife = "10y".parse().unwrap();
        assert!(isclose(t3.value, 10.));
        assert_eq!(t3.unit, TimeUnit::Year);
    }

    #[test]
    fn halflife_to_string() {
        let t1: HalfLife = "1us".parse().unwrap();
        assert_eq!(t1.to_string(), "1μs");

        let t2: HalfLife = "10y".parse().unwrap();
        assert_eq!(t2.to_string(), "10y");
    }

    #[test]
    fn halflife_as_sec() {
        let t1: HalfLife = "1us".parse().unwrap();
        assert!(isclose(t1.as_sec(), 1e-6));

        let t2: HalfLife = "10y".parse().unwrap();
        assert!(isclose(t2.as_sec(), 10. * 365.2422 * 86400.));
    }
}
