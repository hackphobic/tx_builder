use cardano_serialization_lib as csl;

use crate::error::Result;
use crate::value::Assets;

/// An unspent output, the unit of currency the builder operates on.
///
/// Use the constructors to feed in either lovelace alone or lovelace +
/// native assets, plus optionally an inline datum and a reference script.
#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_hash: [u8; 32],
    pub index: u32,
    pub address: csl::Address,
    pub lovelace: u64,
    pub assets: Assets,
    /// Datum sitting inline on the output (CIP-32).
    pub inline_datum: Option<csl::PlutusData>,
    /// Hash of a datum that lives off-chain (the witness must be attached
    /// separately when spent).
    pub datum_hash: Option<csl::DataHash>,
    /// Reference script attached to this output (CIP-33).
    pub script_ref: Option<csl::ScriptRef>,
}

impl Utxo {
    pub fn new(tx_hash: [u8; 32], index: u32, address: csl::Address, lovelace: u64) -> Self {
        Self {
            tx_hash,
            index,
            address,
            lovelace,
            assets: Assets::new(),
            inline_datum: None,
            datum_hash: None,
            script_ref: None,
        }
    }

    pub fn with_assets(mut self, assets: Assets) -> Self {
        self.assets = assets;
        self
    }

    pub fn with_inline_datum(mut self, datum: csl::PlutusData) -> Self {
        self.inline_datum = Some(datum);
        self
    }

    pub fn with_datum_hash(mut self, hash: csl::DataHash) -> Self {
        self.datum_hash = Some(hash);
        self
    }

    pub fn with_script_ref(mut self, script: csl::ScriptRef) -> Self {
        self.script_ref = Some(script);
        self
    }

    pub fn input(&self) -> csl::TransactionInput {
        csl::TransactionInput::new(&csl::TransactionHash::from(self.tx_hash), self.index)
    }

    /// Reconstruct the on-chain `TransactionOutput` corresponding to this UTxO.
    /// Needed when the builder has to know the *value* sitting at an input
    /// (CSL's `TxInputsBuilder` requires this for fee/change calculation).
    pub fn output(&self) -> Result<csl::TransactionOutput> {
        let ma = self.assets.to_csl_multiasset()?;
        let mut value = csl::Value::new(&csl::BigNum::from(self.lovelace));
        if let Some(m) = ma {
            value.set_multiasset(&m);
        }
        let mut out = csl::TransactionOutput::new(&self.address, &value);
        if let Some(d) = &self.inline_datum {
            out.set_plutus_data(d);
        } else if let Some(h) = &self.datum_hash {
            out.set_data_hash(h);
        }
        if let Some(s) = &self.script_ref {
            out.set_script_ref(s);
        }
        Ok(out)
    }

    /// CSL's `TransactionUnspentOutput` (input + reconstructed output).
    pub fn to_unspent(&self) -> Result<csl::TransactionUnspentOutput> {
        Ok(csl::TransactionUnspentOutput::new(&self.input(), &self.output()?))
    }
}
