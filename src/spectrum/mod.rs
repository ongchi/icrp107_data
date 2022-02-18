pub(crate) mod ack;
pub(crate) mod bet;
pub(crate) mod nsf;
pub(crate) mod rad;

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use super::nuclide::Nuclide;
use crate::error::Error;
use crate::reader::FileReader;
use rad::*;

#[derive(Debug)]
pub struct NuclideSpectrum<T>(pub HashMap<Nuclide, Vec<T>>);

impl<T> NuclideSpectrum<T> {
    pub fn new<P>(path: P, range: std::ops::Range<usize>) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        T: FromStr<Err = Error>,
    {
        let mut reader = FileReader::new(path.as_ref());
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while reader.read_str(&mut buf)? != 0 {
            let nuclide = (&buf[0..7]).parse()?;
            let records = &buf[range.clone()].replace("\0", " ");
            let records = records
                .trim()
                .parse::<u64>()
                .map_err(|_| Error::InvalidInteger(records.trim().to_string()))?;

            let mut spectrum = vec![];
            for _ in 0..(records) {
                reader.read_str(&mut buf)?;
                spectrum.push(buf.parse()?);
            }
            inner.insert(nuclide, spectrum);
        }

        Ok(Self(inner))
    }
}

#[derive(Debug)]
pub enum Spectrum {
    Radiation {
        r#type: RadiationType,
        // /nt
        r#yield: f64,
        // MeV
        energy: f64,
    },
    Beta {
        energy: f64,
        // number of beta particles per MeV per nuclear transformation
        number: f64,
    },
    AugerCosterKronigElectron {
        r#yield: f64,
        energy: f64,
        transition: String,
    },
    SpontaneousFissionNeutron {
        energy_lower: f64,
        energy_upper: f64,
        r#yield: f64,
    },
}
