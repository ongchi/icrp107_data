use petgraph::Graph;
use std::collections::HashMap;
use std::path::Path;

use crate::decay_chain;
use crate::error::Error;
use crate::ndx::Attribute;
use crate::reader::{IndexReader, SpectrumReader};
use crate::spectrum::{ack, bet, nsf, rad};
use crate::Nuclide;

#[derive(Debug)]
pub struct NuclideData {
    pub ndx: HashMap<Nuclide, Attribute>,
    pub rad: HashMap<Nuclide, Vec<rad::RadSpectrum>>,
    pub bet: HashMap<Nuclide, Vec<bet::BetSpectrum>>,
    pub ack: HashMap<Nuclide, Vec<ack::AckSpectrum>>,
    pub nsf: HashMap<Nuclide, Vec<nsf::NsfSpectrum>>,
}

impl NuclideData {
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            ndx: IndexReader::new(&path.as_ref().join("ICRP-07.NDX")).read()?,
            rad: SpectrumReader::new(&path.as_ref().join("ICRP-07.RAD")).read()?,
            bet: SpectrumReader::new(&path.as_ref().join("ICRP-07.BET")).read()?,
            ack: SpectrumReader::new(&path.as_ref().join("ICRP-07.ACK")).read()?,
            nsf: SpectrumReader::new(&path.as_ref().join("ICRP-07.NSF")).read()?,
        })
    }

    pub fn get_decay_chain_graph(
        &self,
        nuclide: Nuclide,
    ) -> Graph<decay_chain::Node, decay_chain::Edge> {
        let mut graph = Graph::new();
        let mut edges = Vec::new();
        decay_chain::build_graph(self, &mut graph, &mut edges, nuclide);
        graph.extend_with_edges(edges.iter());

        graph
    }
}
