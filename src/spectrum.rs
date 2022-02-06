use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use super::nuclide::Nuclide;
use super::{ack, bet, nsf, rad, FileReader, ParseError};

pub struct SpectrumEntry {
    pub nuclide: Nuclide,
    pub records: u64,
}

impl From<rad::Entry> for SpectrumEntry {
    fn from(entry: rad::Entry) -> Self {
        Self {
            nuclide: entry.nuclide,
            records: entry.records,
        }
    }
}

impl From<bet::Entry> for SpectrumEntry {
    fn from(entry: bet::Entry) -> Self {
        Self {
            nuclide: entry.nuclide,
            records: entry.records,
        }
    }
}

impl From<ack::Entry> for SpectrumEntry {
    fn from(entry: ack::Entry) -> Self {
        Self {
            nuclide: entry.nuclide,
            records: entry.records,
        }
    }
}

impl From<nsf::Entry> for SpectrumEntry {
    fn from(entry: nsf::Entry) -> Self {
        Self {
            nuclide: entry.nuclide,
            records: entry.records,
        }
    }
}

#[derive(Debug)]
pub struct NuclideSpectrum<T>(pub HashMap<Nuclide, Vec<T>>);

impl<T: FromStr> NuclideSpectrum<T> {
    pub fn new<P, R>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
        R: FromStr<Err = ParseError> + Into<SpectrumEntry>,
        T: FromStr<Err = ParseError>,
    {
        let mut reader = FileReader::new(path.as_ref());
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while reader.read_buf(&mut buf)? != 0 {
            let record: SpectrumEntry = buf.parse::<R>()?.into();

            let mut spectrum = vec![];
            for _ in 0..(record.records) {
                reader.read_buf(&mut buf)?;
                spectrum.push(buf.parse()?);
            }
            inner.insert(record.nuclide, spectrum);
        }

        Ok(Self(inner))
    }
}

#[derive(Debug)]
pub enum Spectrum {
    Radiation(rad::Spectrum),
    Beta(bet::Spectrum),
    AugerCkElectron(ack::Spectrum),
    Neutron(nsf::Spectrum),
}
