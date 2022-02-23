use petgraph::{graph::NodeIndex, Graph};

use crate::{DecayMode, HalfLife, Nuclide, NuclideData};

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
    decay_mode: DecayMode,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.branch_rate, self.decay_mode)
    }
}

pub(crate) fn build_graph(
    data: &NuclideData,
    graph: &mut Graph<Node, Edge>,
    edges: &mut Vec<(NodeIndex, NodeIndex, Edge)>,
    nuclide: Nuclide,
) {
    macro_rules! get_or_add_node_index {
        ($nuc:expr) => {{
            match graph
                .raw_nodes()
                .iter()
                .position(|n| n.weight.nuclide == $nuc)
            {
                Some(i) => NodeIndex::new(i),
                None => graph.add_node(Node {
                    nuclide: $nuc,
                    half_life: match data.ndx.get(&$nuc) {
                        Some(attr) => Some(attr.half_life),
                        None => None,
                    },
                }),
            }
        }};
    }

    let parent = get_or_add_node_index!(nuclide);

    match &data.ndx.get(&nuclide) {
        Some(attr) => {
            for d in attr.progeny.iter() {
                let progeny = get_or_add_node_index!(d.nuclide);

                if edges
                    .iter()
                    .position(|(p, d, _)| {
                        p.index() == parent.index() && d.index() == progeny.index()
                    })
                    .is_none()
                {
                    let attr = Edge {
                        branch_rate: d.branch_rate,
                        decay_mode: d.mode,
                    };
                    edges.push((parent, progeny, attr));
                }

                build_graph(data, graph, edges, d.nuclide);
            }
        }
        None => {}
    }
}
