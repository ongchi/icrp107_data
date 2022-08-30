use crate::primitive::attr::Energy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid atomic number: {0}")]
    InvalidAtomicNumber(u8),
    #[error("invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("invalid nuclide: {0}")]
    InvalidNuclide(String),
    #[error("invalid half life: {0}")]
    InvalidHalfLife(String),
    #[error("invalid decay mode: {0}")]
    InvalidDecayMode(String),
    #[error("invalid radiation type: {0}")]
    InvalidRadiationType(String),
    #[error("invalid float number: {0}")]
    InvalidFloat(String),
    #[error("invalid integer: {0}")]
    InvalidInteger(String),
    #[error("invalid energy: {0}")]
    InvalidEnergy(Energy),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        e.into()
    }
}
