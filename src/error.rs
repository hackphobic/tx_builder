use cardano_serialization_lib::{DeserializeError, JsError};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// Anything raised by `cardano-serialization-lib` itself.
    #[error("cardano-serialization-lib: {0}")]
    Csl(String),

    #[error("deserialize: {0}")]
    Deserialize(String),

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid asset name (must be <= 32 bytes): {0}")]
    InvalidAssetName(String),

    #[error("invalid policy id (must be 28 bytes): {0}")]
    InvalidPolicyId(String),

    #[error("missing change address — call `.change_address(..)` before `.build()`")]
    MissingChangeAddress,

    #[error("Plutus input requires a redeemer")]
    MissingRedeemer,

    #[error(
        "Plutus input requires a datum source (inline-on-utxo, witness-attached, or hash-only)"
    )]
    MissingDatum,

    #[error("transaction has Plutus scripts but no collateral inputs were supplied")]
    MissingCollateral,

    #[error("output below min-ada: {0}")]
    BelowMinAda(String),

    #[error("invalid script: {0}")]
    InvalidScript(String),

    #[error("invalid datum: {0}")]
    InvalidDatum(String),

    #[error("invalid redeemer: {0}")]
    InvalidRedeemer(String),

    #[error("{0}")]
    Custom(String),
}

impl From<JsError> for Error {
    fn from(e: JsError) -> Self {
        Error::Csl(e.to_string())
    }
}

impl From<DeserializeError> for Error {
    fn from(e: DeserializeError) -> Self {
        Error::Deserialize(e.to_string())
    }
}
