mod graph;

pub use graph::{DecayChain, DecayChainBuilder};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::primitive::attr::{DecayConstant, NuclideProgeny};
use crate::primitive::Nuclide;

#[derive(Debug)]
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

type CachedData = BTreeMap<Nuclide, Vec<(Vec<f64>, Vec<f64>)>>;

pub struct BatemanDecaySolver<'a, D>
where
    D: NuclideProgeny + DecayConstant,
{
    decay_data: &'a D,
    cache: RefCell<BTreeMap<Nuclide, Rc<CachedData>>>,
}

impl<'a, D> BatemanDecaySolver<'a, D>
where
    D: NuclideProgeny + DecayConstant,
{
    pub fn new(decay_data: &'a D) -> Self {
        Self {
            decay_data,
            cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Decay calculation for decay_time in seconds.
    pub fn decay(&self, inventory: &Inventory, decay_time: u64) -> Inventory {
        let mut new_inv = Inventory::new();

        for (&nuclide, &activity) in inventory.iter() {
            if let Some(decay_data) = self.get_decay_data(nuclide) {
                let n0 = activity / self.decay_data.lambda(nuclide).unwrap();
                for (nuc, a) in self.bateman_eq(&decay_data, n0, decay_time) {
                    new_inv.add(nuc, a);
                }
            }
        }

        new_inv
    }

    fn bateman_eq(&self, decay_data: &CachedData, n0: f64, t: u64) -> BTreeMap<Nuclide, f64> {
        let mut res = BTreeMap::new();
        for (nuc, data) in decay_data {
            for (br, lamb) in data {
                *res.entry(*nuc).or_insert(0.) += n0
                    * (lamb.iter().product::<f64>() * br.iter().product::<f64>())
                    * (lamb.iter().map(|&li| {
                        (-li * (t as f64)).exp()
                            / (lamb.iter().map(|&lj| lj - li))
                                .filter(|&l| l != 0.)
                                .product::<f64>()
                    }))
                    .sum::<f64>();
            }
        }

        res
    }

    fn get_decay_data(&self, parent: Nuclide) -> Option<Rc<CachedData>> {
        Some(
            self.cache
                .borrow_mut()
                .entry(parent)
                .or_insert({
                    let mut stack =
                        vec![(parent, vec![], vec![self.decay_data.lambda(parent).ok()?])];

                    let mut decay_data = BTreeMap::new();

                    while let Some((parent, br, lambda)) = stack.pop() {
                        decay_data.entry(parent).or_insert(vec![]).push((
                            br.iter()
                                .take(std::cmp::max(0, br.len() as i32 - 1) as usize)
                                .copied()
                                .collect(),
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

                    Rc::new(decay_data)
                })
                .clone(),
        )
    }
}
