use cardano_serialization_lib as csl;

use crate::utxo::Utxo;

/// Where the Plutus / native script being witnessed comes from.
///
/// Cardano supports two attachment modes:
/// - **`Provided`** — the script is included in the witness set of *this*
///   transaction. Costs more fee (bytes).
/// - **`Reference`** — the script lives on-chain inside another UTxO's
///   `script_ref` (CIP-33). The tx only carries a reference input pointing
///   at it.
#[derive(Debug, Clone)]
pub enum ScriptSource {
    /// Provide the full Plutus script bytes in this transaction.
    Provided(csl::PlutusScript),
    /// Refer to a script attached to an on-chain UTxO. The referenced UTxO
    /// will be automatically added as a *reference input* on build.
    Reference {
        utxo: Box<Utxo>,
        script_hash: csl::ScriptHash,
        language: csl::Language,
        /// Size in bytes of the referenced script — required for the
        /// reference-script fee in Conway.
        script_size: u64,
    },
}

impl ScriptSource {
    /// CSL's `PlutusScriptSource`, which wraps both variants.
    pub fn to_plutus_script_source(&self) -> csl::PlutusScriptSource {
        match self {
            ScriptSource::Provided(script) => csl::PlutusScriptSource::new(script),
            ScriptSource::Reference {
                utxo,
                script_hash,
                language,
                script_size,
            } => csl::PlutusScriptSource::new_ref_input(
                script_hash,
                &utxo.input(),
                language,
                *script_size as usize,
            ),
        }
    }
}

/// Where the datum needed to spend a Plutus UTxO comes from.
#[derive(Debug, Clone)]
pub enum DatumSpec {
    /// The datum is inline on the UTxO being spent — nothing to attach.
    Inline,
    /// The datum is attached as a witness in this transaction.
    Witness(csl::PlutusData),
    /// Datum lives in another reference input (rare; CIP-33 reference
    /// scripts more common). Provide the referenced UTxO.
    Reference(Box<Utxo>),
}
