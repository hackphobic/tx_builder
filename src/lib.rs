//! # cardano-tx-builder
//!
//! Ergonomic transaction builder on top of
//! [`cardano-serialization-lib`](https://docs.rs/cardano-serialization-lib)
//! (CSL), focused on dApp / Plutus script interactions and Conway-era
//! governance.
//!
//! ## What this is
//!
//! - A fluent wrapper around CSL's `TransactionBuilder`, `MintBuilder`,
//!   `CertificatesBuilder`, `VotingBuilder`, `VotingProposalBuilder`,
//!   `WithdrawalsBuilder`, etc., that hides most of the `BigNum` / index
//!   plumbing.
//! - Designed for callers that already have selected UTxOs and just want to
//!   shape and serialize a transaction.
//!
//! ## What this isn't
//!
//! - **No coin selection.** Bring your own inputs.
//! - **No signing.** [`TxBuilder::build`] returns an unsigned [`csl::Transaction`]
//!   (with the witness set populated for scripts/datums/redeemers, but no
//!   vkey witnesses). Sign it externally and attach `Vkeywitness` entries.
//! - **No network I/O.** Fetch protocol params / UTxOs however you like.
//!
//! ## Quick start
//!
//! ```no_run
//! use cardano_tx_builder::prelude::*;
//!
//! # fn demo(params: ProtocolParameters, change: csl::Address, utxo: Utxo,
//! #         dest: csl::Address) -> Result<()> {
//! let tx = TxBuilder::new(&params)
//!     .change_address(&change)
//!     .add_pubkey_input(&utxo)
//!     .add_output(Output::new(&dest).lovelace(2_000_000))
//!     .build()?;
//!
//! // `tx` is unsigned; serialize and pass to whatever holds your keys.
//! let cbor_hex = hex::encode(tx.to_bytes());
//! # let _ = cbor_hex; Ok(())
//! # }
//! ```

pub mod builder;
pub mod cert;
pub mod config;
pub mod error;
pub mod governance;
pub mod input;
pub mod mint;
pub mod output;
pub mod script;
pub mod utxo;
pub mod value;

pub use builder::TxBuilder;
pub use config::ProtocolParameters;
pub use error::{Error, Result};
pub use input::SpendingInput;
pub use mint::MintAction;
pub use output::Output;
pub use script::{DatumSpec, ScriptSource};
pub use utxo::Utxo;

/// Re-export of `cardano-serialization-lib` so callers don't have to pin it
/// separately.
pub use cardano_serialization_lib as csl;

pub mod prelude {
    //! Glob-importable everyday types.
    pub use crate::cert::Cert;
    pub use crate::governance::{ProposalSpec, Vote, VoterSpec};
    pub use crate::{
        csl, DatumSpec, Error, MintAction, Output, ProtocolParameters, Result, ScriptSource,
        SpendingInput, TxBuilder, Utxo,
    };
}
