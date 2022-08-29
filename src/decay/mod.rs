mod graph;

pub use graph::{DecayChain, DecayChainBuilder};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::error::Error;
use crate::nuclide::{HalfLife, Nuclide, Progeny};

pub trait DecayData {
    fn check_nuclide(&self, nuclide: Nuclide) -> Result<(), Error>;

    fn progeny(&self, nuclide: Nuclide) -> Result<&[Progeny], Error>;

    fn half_life(&self, nuclide: Nuclide) -> Result<HalfLife, Error>;

    fn lambda(&self, nuclide: Nuclide) -> Result<f64, Error>;
}

#[derive(Debug)]
pub struct Inventory(BTreeMap<Nuclide, f64>);

impl Deref for Inventory {
    type Target = BTreeMap<Nuclide, f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct InventoryBuilder<'a> {
    data: &'a dyn DecayData,
    inner: Inventory,
}

impl<'a> InventoryBuilder<'a> {
    pub fn new(data: &'a dyn DecayData) -> Self {
        Self {
            data,
            inner: Inventory(BTreeMap::new()),
        }
    }

    pub fn add(self, nuclide: Nuclide, activity: f64) -> Result<Self, Error> {
        self.data.check_nuclide(nuclide)?;

        Ok(self.add_unchecked(nuclide, activity))
    }

    pub fn add_unchecked(mut self, nuclide: Nuclide, activity: f64) -> Self {
        let inner = &mut self.inner.0;
        *inner.entry(nuclide).or_insert(0.0) += activity;

        self
    }

    pub fn remove(mut self, nuclide: Nuclide) -> Result<Self, Error> {
        self.inner.0.remove_entry(&nuclide);

        Ok(self)
    }

    pub fn zero_out(mut self) -> Self {
        for (_, a) in self.inner.0.iter_mut() {
            *a = 0.;
        }

        self
    }

    pub fn build(self) -> Inventory {
        self.inner
    }
}

type CachedData = BTreeMap<Nuclide, Vec<(Vec<f64>, Vec<f64>)>>;

pub struct BatemanDecaySolver<'a> {
    decay_data: &'a dyn DecayData,
    cache: RefCell<BTreeMap<Nuclide, Rc<CachedData>>>,
}

impl<'a> BatemanDecaySolver<'a> {
    pub fn new(decay_data: &'a dyn DecayData) -> Self {
        Self {
            decay_data,
            cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Decay calculation for decay_time in seconds.
    pub fn decay(&self, inventory: &Inventory, decay_time: u64) -> Inventory {
        let mut inv_factory = InventoryBuilder::new(self.decay_data);

        for (&nuclide, &activity) in inventory.iter() {
            let decay_data = self.get_decay_data(nuclide);
            let n0 = activity / self.decay_data.lambda(nuclide).unwrap();
            for (nuc, a) in self.bateman_eq(&decay_data, n0, decay_time) {
                inv_factory = inv_factory.add_unchecked(nuc, a);
            }
        }

        inv_factory.build()
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

    fn get_decay_data(&self, parent: Nuclide) -> Rc<CachedData> {
        self.cache
            .borrow_mut()
            .entry(parent)
            .or_insert({
                let mut stack = vec![(
                    parent,
                    vec![],
                    vec![self.decay_data.lambda(parent).unwrap()],
                )];

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
                        if self.decay_data.check_nuclide(daughter.nuclide).is_ok() {
                            let mut lambda = lambda.clone();
                            lambda.push(self.decay_data.lambda(daughter.nuclide).unwrap());
                            stack.push((daughter.nuclide, br, lambda));
                        }
                    }
                }

                Rc::new(decay_data)
            })
            .clone()
    }
}
