#[derive(thiserror::Error, Debug)]
pub enum Error {
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
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
