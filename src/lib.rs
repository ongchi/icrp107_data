pub mod atten_coef;
pub mod dataset;
pub mod decay;
mod error;
mod macros;
pub mod molecular;
pub mod nuclide;

pub use atten_coef::{AttenCoefData, Material};
pub use dataset::{Icrp107, NistMassAttenCoef};
pub use decay::{
    BatemanDecaySolver, DecayChain, DecayChainBuilder, DecayData, Inventory, InventoryBuilder,
};
pub use nuclide::{DecayMode, DecayModePrimitive, HalfLife, Nuclide};
