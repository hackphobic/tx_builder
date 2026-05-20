use cardano_serialization_lib as csl;

use crate::script::{DatumSpec, ScriptSource};
use crate::utxo::Utxo;

/// How an input is being spent.
#[derive(Debug, Clone)]
pub enum SpendingInput {
    /// UTxO locked by a public-key credential. Will be witnessed by a vkey
    /// signature attached *after* building.
    Pubkey(Utxo),

    /// UTxO locked by a Plutus script.
    Plutus {
        utxo: Utxo,
        script: ScriptSource,
        datum: DatumSpec,
        redeemer: csl::PlutusData,
        /// Initial ex_units estimate. Replace with a real budget from an
        /// ex-units evaluator (Ogmios `evaluateTransaction`, Blockfrost
        /// `tx/utxos`, etc.) before signing for real.
        ex_units: csl::ExUnits,
    },

    /// UTxO locked by a native script.
    NativeScript {
        utxo: Utxo,
        script: csl::NativeScript,
    },
}

impl SpendingInput {
    /// Convenience constructor for the most common case.
    pub fn pubkey(utxo: Utxo) -> Self {
        SpendingInput::Pubkey(utxo)
    }

    pub fn plutus(
        utxo: Utxo,
        script: ScriptSource,
        datum: DatumSpec,
        redeemer: csl::PlutusData,
        ex_units: csl::ExUnits,
    ) -> Self {
        SpendingInput::Plutus {
            utxo,
            script,
            datum,
            redeemer,
            ex_units,
        }
    }

    pub fn native(utxo: Utxo, script: csl::NativeScript) -> Self {
        SpendingInput::NativeScript { utxo, script }
    }

    pub fn utxo(&self) -> &Utxo {
        match self {
            SpendingInput::Pubkey(u) => u,
            SpendingInput::Plutus { utxo, .. } => utxo,
            SpendingInput::NativeScript { utxo, .. } => utxo,
        }
    }

    pub(crate) fn is_plutus(&self) -> bool {
        matches!(self, SpendingInput::Plutus { .. })
    }
}
