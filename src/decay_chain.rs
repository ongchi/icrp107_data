use std::collections::HashSet;

use petgraph::{graph::NodeIndex, Graph};

use crate::nuclide::{DecayMode, HalfLife, Nuclide, Progeny};
use flagset::FlagSet;

#[derive(Default, Clone, Copy)]
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
    decay_mode: FlagSet<DecayMode>,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.branch_rate)?;
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

    fn build_graph<N: Into<Nuclide>>(&self, parent: N) -> Graph<Node, Edge> {
        let mut graph = Graph::new();
        let mut edges = vec![];

        let mut get_or_insert_node = |nuclide: Nuclide| -> NodeIndex {
            match graph
                .raw_nodes()
                .iter()
                .position(|n: &petgraph::graph::Node<Node>| n.weight.nuclide == nuclide)
            {
                Some(i) => NodeIndex::new(i),
                None => {
                    let half_life = self.get_half_life(&nuclide);
                    graph.add_node(Node { nuclide, half_life })
                }
            }
        };

        let mut stack = vec![parent.into()];
        let mut visited: HashSet<usize> = HashSet::new();
        while !stack.is_empty() {
            let p = stack.pop().unwrap();
            let p_node = get_or_insert_node(p);

            if visited.contains(&p_node.index()) {
                continue;
            } else {
                visited.insert(p_node.index());
            }

            match self.get_progeny(&p) {
                Some(progeny) => {
                    for d in progeny {
                        stack.push(d.nuclide);
                        let d_node = get_or_insert_node(d.nuclide);
                        let edge = Edge {
                            branch_rate: d.branch_rate,
                            decay_mode: d.decay_mode,
                        };
                        edges.push((p_node, d_node, edge));
                    }
                }
                None => {}
            }
        }
        graph.extend_with_edges(edges);

        graph
    }
}
