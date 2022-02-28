use once_cell::sync::OnceCell;
use paste::paste;
use petgraph::Graph;
use std::collections::HashMap;
use std::path::Path;

use crate::decay_chain;
use crate::error::Error;
use crate::ndx;
use crate::reader::{IndexReader, SpectrumReader};
use crate::spectrum::{ack, bet, nsf, rad};
use crate::Nuclide;

pub struct NuclideData {
    ndx: IndexReader,
    rad: SpectrumReader<rad::RadSpectrum>,
    bet: SpectrumReader<bet::BetSpectrum>,
    ack: SpectrumReader<ack::AckSpectrum>,
    nsf: SpectrumReader<nsf::NsfSpectrum>,
}

macro_rules! data_getter {
    ($name:ident, $val:ty) => {
        paste! {
            pub fn $name(&mut self) -> Result<&HashMap<Nuclide, $val>, Error> {
                static [<$name:upper>]: OnceCell<HashMap<Nuclide, $val>> = OnceCell::new();
                [<$name:upper>].get_or_try_init(|| self.$name.read())
            }
        }
    };
}

impl NuclideData {
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            ndx: IndexReader::new(&path.as_ref().join("ICRP-07.NDX")),
            rad: SpectrumReader::new(&path.as_ref().join("ICRP-07.RAD")),
            bet: SpectrumReader::new(&path.as_ref().join("ICRP-07.BET")),
            ack: SpectrumReader::new(&path.as_ref().join("ICRP-07.ACK")),
            nsf: SpectrumReader::new(&path.as_ref().join("ICRP-07.NSF")),
        })
    }

    data_getter!(ndx, ndx::Attribute);
    data_getter!(rad, Vec<rad::RadSpectrum>);
    data_getter!(bet, Vec<bet::BetSpectrum>);
    data_getter!(ack, Vec<ack::AckSpectrum>);
    data_getter!(nsf, Vec<nsf::NsfSpectrum>);

    pub fn get_decay_chain_graph(
        &mut self,
        nuclide: Nuclide,
    ) -> Result<Graph<decay_chain::Node, decay_chain::Edge>, Error> {
        let mut graph = Graph::new();
        let mut edges = Vec::new();
        decay_chain::build_graph(self.ndx()?, &mut graph, &mut edges, nuclide)?;
        graph.extend_with_edges(edges.iter());

        Ok(graph)
    }
}
