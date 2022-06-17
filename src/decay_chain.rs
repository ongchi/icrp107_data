use float_pretty_print::PrettyPrintFloat;
use petgraph::{graph::NodeIndex, Graph};
use std::collections::HashSet;

use crate::nuclide::{DecayMode, HalfLife, MaybeNuclide, Nuclide, Progeny};

#[derive(Clone, Copy)]
pub struct Node {
    nuclide: MaybeNuclide,
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
    decay_mode: DecayMode,
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

pub trait DecayChain {
    fn get_progeny(&self, nuclide: &Nuclide) -> Option<Vec<Progeny>>;

    fn get_half_life(&self, nuclide: &Nuclide) -> Option<HalfLife>;

    fn build_graph<N: Into<MaybeNuclide>>(&self, root: N) -> Graph<Node, Edge> {
        let mut graph: Graph<Node, Edge> = Graph::new();

        let mut get_or_insert_node = |nuclide: MaybeNuclide| -> NodeIndex {
            match nuclide {
                MaybeNuclide::Nuclide(nuc) => {
                    match graph
                        .raw_nodes()
                        .iter()
                        .position(|n| n.weight.nuclide == nuclide)
                    {
                        Some(i) => NodeIndex::new(i),
                        None => {
                            let half_life = self.get_half_life(&nuc);
                            graph.add_node(Node { nuclide, half_life })
                        }
                    }
                }
                MaybeNuclide::SF => {
                    let half_life = None;
                    graph.add_node(Node { nuclide, half_life })
                }
            }
        };

        let mut stack: Vec<MaybeNuclide> = vec![root.into()];
        let mut visited = HashSet::new();
        let mut edges = vec![];
        while !stack.is_empty() {
            match stack.pop().unwrap() {
                parent @ MaybeNuclide::Nuclide(nuc) => {
                    visited.insert(nuc);

                    match self.get_progeny(&nuc) {
                        Some(progeny) => {
                            let p_node = get_or_insert_node(parent);
                            for daughter in progeny {
                                {
                                    if let MaybeNuclide::Nuclide(d) = daughter.nuclide {
                                        if !visited.contains(&d) {
                                            stack.push(daughter.nuclide)
                                        }
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
                MaybeNuclide::SF => {}
            }
        }
        for (p_node, d_node, weight) in edges {
            graph.add_edge(p_node, d_node, weight);
        }

        graph
    }
}
