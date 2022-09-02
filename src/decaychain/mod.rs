mod graph;

pub use graph::{DecayChain, DecayChainBuilder};

use std::cmp::max;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::primitive::attr::{DecayConstant, NuclideProgeny};
use crate::primitive::Nuclide;

#[derive(Debug, Clone)]
pub struct Inventory(BTreeMap<Nuclide, f64>);

impl Deref for Inventory {
    type Target = BTreeMap<Nuclide, f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add(&mut self, nuclide: Nuclide, activity: f64) {
        let inner = &mut self.0;
        *inner.entry(nuclide).or_insert(0.0) += activity;
    }

    pub fn remove(&mut self, nuclide: Nuclide) -> Option<(Nuclide, f64)> {
        self.0.remove_entry(&nuclide)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

type CachedNode = Arc<BTreeMap<Nuclide, Vec<(Vec<f64>, Vec<f64>)>>>;
type CachedData = RwLock<BTreeMap<Nuclide, CachedNode>>;

pub struct BatemanDecaySolver<D> {
    decay_data: Arc<D>,
    cache: CachedData,
}

impl<D> BatemanDecaySolver<D>
where
    D: NuclideProgeny + DecayConstant,
{
    pub fn new(decay_data: Arc<D>) -> Arc<Self> {
        Arc::new(Self {
            decay_data,
            cache: RwLock::new(BTreeMap::new()),
        })
    }

    /// Decay calculation for decay_time in seconds.
    pub fn decay(&self, inventory: &Inventory, decay_time: f64) -> Inventory {
        let mut inv = Inventory::new();

        for (&nuclide, &activity) in inventory.iter() {
            if let Some(bateman_res) = self.bateman_eq(nuclide, decay_time) {
                let n0 = activity / self.decay_data.lambda(nuclide).unwrap();
                for (nuc, lamb) in bateman_res {
                    inv.add(nuc, lamb * n0);
                }
            }
        }

        inv
    }

    // Bateman Equation
    pub fn bateman_eq(&self, nuclide: Nuclide, dt: f64) -> Option<BTreeMap<Nuclide, f64>> {
        if let Some(decay_vars) = self.decay_vars(nuclide) {
            let mut res = BTreeMap::new();
            for (&nuc, cache) in decay_vars.iter() {
                for (br, lamb) in cache {
                    let val: f64 = lamb.iter().chain(br.iter()).product();
                    let val = val
                        * (lamb.iter())
                            .map(|li| {
                                (-li * dt).exp()
                                    / (lamb.iter().map(|lj| lj - li))
                                        .filter(|v| v != &0.)
                                        .product::<f64>()
                            })
                            .sum::<f64>();
                    *res.entry(nuc).or_insert(0.) += val;
                }
            }

            Some(res)
        } else {
            None
        }
    }

    // Variables for calculate with Bateman Equation
    fn decay_vars(&self, parent: Nuclide) -> Option<CachedNode> {
        let cache = self.cache.read().unwrap();

        if let Some(decay_vars) = cache.get(&parent) {
            Some(decay_vars.clone())
        } else {
            drop(cache);

            let mut stack = vec![(parent, vec![], vec![self.decay_data.lambda(parent).ok()?])];
            let mut decay_vars = BTreeMap::new();

            while let Some((parent, br, lambda)) = stack.pop() {
                decay_vars.entry(parent).or_insert(vec![]).push((
                    br.iter().take(max(0usize, br.len() - 1)).copied().collect(),
                    lambda.clone(),
                ));

                for daughter in self.decay_data.progeny(parent).unwrap() {
                    let mut br = br.clone();
                    br.push(daughter.branch_rate);
                    let mut lambda = lambda.clone();
                    if let Ok(lambda_d) = self.decay_data.lambda(daughter.nuclide) {
                        lambda.push(lambda_d);
                        stack.push((daughter.nuclide, br, lambda));
                    }
                }
            }

            let mut cache = self.cache.write().unwrap();
            let decay_vars = Arc::new(decay_vars);
            cache.insert(parent, decay_vars.clone());

            Some(decay_vars)
        }
    }
}
