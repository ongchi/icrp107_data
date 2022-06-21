mod graph;

pub use graph::{DecayChain, DecayChainBuilder};

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::Error;
use crate::nuclide::{HalfLife, Nuclide, Progeny};

pub trait DecayData {
    fn check_nuclide(&self, nuclide: &Nuclide) -> Result<(), Error>;

    fn progeny(&self, nuclide: &Nuclide) -> Result<&[Progeny], Error>;

    fn half_life(&self, nuclide: &Nuclide) -> Result<HalfLife, Error>;

    fn lambda(&self, nuclide: &Nuclide) -> Result<f64, Error>;
}

type Inventory = BTreeMap<Nuclide, f64>;

pub struct InventoryFactory {
    inner: Inventory,
    data: Arc<dyn DecayData>,
}

impl InventoryFactory {
    pub fn new(data: Arc<dyn DecayData>) -> Self {
        Self {
            inner: Inventory::new(),
            data,
        }
    }

    pub fn insert<N: Into<Nuclide>>(&mut self, nuclide: N, activity: f64) -> Result<(), Error> {
        let nuclide = nuclide.into();
        self.data.check_nuclide(&nuclide)?;
        self.inner.insert(nuclide, activity);

        Ok(())
    }

    pub fn remove<N: Into<Nuclide>>(&mut self, nuclide: N) -> Result<(), Error> {
        let nuclide = nuclide.into();
        self.inner.remove_entry(&nuclide);

        Ok(())
    }

    pub fn export(&mut self) -> Inventory {
        let export = self.inner.clone();
        self.inner.clear();

        export
    }
}
