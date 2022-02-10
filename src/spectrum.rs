use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use super::nuclide::Nuclide;
use super::{ack, bet, nsf, rad, FileReader, ParseError};

#[derive(Debug)]
pub struct NuclideSpectrum<T>(pub HashMap<Nuclide, Vec<T>>);

impl<T> NuclideSpectrum<T> {
    pub fn new<P>(path: P, range: std::ops::Range<usize>) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
        T: FromStr<Err = ParseError>,
    {
        let mut reader = FileReader::new(path.as_ref());
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while reader.read_buf(&mut buf)? != 0 {
            let nuclide = &buf[0..7];
            let nuclide = nuclide.parse()?;
            let records = &buf[range.clone()];
            let records = records
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .parse::<u64>()
                .map_err(|_| ParseError::InvalidInteger(records.trim().to_string()))?;

            let mut spectrum = vec![];
            for _ in 0..(records) {
                reader.read_buf(&mut buf)?;
                spectrum.push(buf.parse()?);
            }
            inner.insert(nuclide, spectrum);
        }

        Ok(Self(inner))
    }
}

#[derive(Debug)]
pub enum Spectrum {
    Radiation(rad::Spectrum),
    Beta(bet::Spectrum),
    AugerCKElectron(ack::Spectrum),
    Neutron(nsf::Spectrum),
}
