use cardano_serialization_lib as csl;

use crate::error::Result;
use crate::value::{csl_value, Assets};

/// Ergonomic output description.
///
/// `Output::new(addr).lovelace(2_000_000).asset(...)` etc. then converted
/// to a `TransactionOutput` at build time.
#[derive(Debug, Clone)]
pub struct Output {
    pub address: csl::Address,
    pub lovelace: u64,
    pub assets: Assets,
    pub datum: OutputDatum,
    pub script_ref: Option<csl::ScriptRef>,
}

/// Datum attached to an output.
#[derive(Debug, Clone, Default)]
pub enum OutputDatum {
    #[default]
    None,
    /// Just the hash — the actual datum lives off-chain.
    Hash(csl::DataHash),
    /// CIP-32 inline datum.
    Inline(csl::PlutusData),
}

impl Output {
    pub fn new(address: &csl::Address) -> Self {
        Self {
            address: address.clone(),
            lovelace: 0,
            assets: Assets::new(),
            datum: OutputDatum::None,
            script_ref: None,
        }
    }

    pub fn lovelace(mut self, amount: u64) -> Self {
        self.lovelace = amount;
        self
    }

    pub fn assets(mut self, assets: Assets) -> Self {
        self.assets = assets;
        self
    }

    pub fn inline_datum(mut self, d: csl::PlutusData) -> Self {
        self.datum = OutputDatum::Inline(d);
        self
    }

    pub fn datum_hash(mut self, h: csl::DataHash) -> Self {
        self.datum = OutputDatum::Hash(h);
        self
    }

    pub fn script_ref(mut self, s: csl::ScriptRef) -> Self {
        self.script_ref = Some(s);
        self
    }

    pub fn to_csl(&self) -> Result<csl::TransactionOutput> {
        let ma = self.assets.to_csl_multiasset()?;
        let value = csl_value(self.lovelace, ma.as_ref());
        let mut out = csl::TransactionOutput::new(&self.address, &value);
        match &self.datum {
            OutputDatum::None => {}
            OutputDatum::Hash(h) => out.set_data_hash(h),
            OutputDatum::Inline(d) => out.set_plutus_data(d),
        }
        if let Some(s) = &self.script_ref {
            out.set_script_ref(s);
        }
        Ok(out)
    }
}
