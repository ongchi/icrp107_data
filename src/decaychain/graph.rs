use std::collections::HashSet;
use std::sync::Arc;

use float_pretty_print::PrettyPrintFloat;
use petgraph::{graph::NodeIndex, Graph};

use crate::primitive::attr::{NuclideHalfLife, NuclideProgeny};
use crate::primitive::{DecayModeFlagSet, HalfLife, Nuclide};

#[derive(Clone, Copy)]
pub struct Node {
    nuclide: Nuclide,
    half_life: Option<HalfLife>,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}",
            self.nuclide,
            match self.half_life {
                Some(t) => t.to_string(),
                None => "".to_string(),
            }
        )
    }
}

#[derive(Clone)]
pub struct Edge {
    branch_rate: f64,
    decay_mode: DecayModeFlagSet,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", PrettyPrintFloat(self.branch_rate))?;
        for (i, mode) in self.decay_mode.into_iter().enumerate() {
            if i == 0 {
                write!(f, "{}", mode)?;
            } else {
                write!(f, " or {}", mode)?;
            }
        }
        write!(f, "")
    }
}

pub type DecayChain = Graph<Node, Edge>;

pub struct DecayChainBuilder<D> {
    data: Arc<D>,
}

impl<D> DecayChainBuilder<D>
where
    D: NuclideHalfLife + NuclideProgeny,
{
    pub fn new(data: Arc<D>) -> Self {
        Self { data }
    }

    pub fn build(self, root: Nuclide) -> DecayChain {
        let mut graph: Graph<Node, Edge> = Graph::new();

        let mut get_or_insert_node = |nuclide: Nuclide| -> NodeIndex {
            match nuclide {
                Nuclide::WithId(_) => {
                    match graph
                        .raw_nodes()
                        .iter()
                        .position(|n| n.weight.nuclide == nuclide)
                    {
                        Some(i) => NodeIndex::new(i),
                        None => {
                            let half_life = self.data.half_life(nuclide).ok();
                            graph.add_node(Node { nuclide, half_life })
                        }
                    }
                }
                Nuclide::FissionProducts => {
                    let half_life = None;
                    graph.add_node(Node { nuclide, half_life })
                }
            }
        };

        let mut stack: Vec<Nuclide> = vec![root];
        let mut visited = HashSet::new();
        let mut edges = vec![];

        while let Some(parent) = stack.pop() {
            match parent {
                Nuclide::WithId(_) => {
                    visited.insert(parent);

                    match self.data.progeny(parent).ok() {
                        Some(progeny) => {
                            let p_node = get_or_insert_node(parent);
                            for daughter in progeny {
                                {
                                    if !visited.contains(&daughter.nuclide) {
                                        stack.push(daughter.nuclide)
                                    }

                                    let d_node = get_or_insert_node(daughter.nuclide);
                                    let weight = Edge {
                                        branch_rate: daughter.branch_rate,
                                        decay_mode: daughter.decay_mode,
                                    };
                                    edges.push((p_node, d_node, weight));
                                }
                            }
                        }
                        None => {}
                    };
                }
                Nuclide::FissionProducts => {}
            }
        }
        for (p_node, d_node, weight) in edges {
            graph.add_edge(p_node, d_node, weight);
        }

        graph
    }
}
