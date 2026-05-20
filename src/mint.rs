use cardano_serialization_lib as csl;

use crate::script::ScriptSource;
use crate::value::{AssetName, PolicyId};

/// A single mint or burn (negative quantity = burn).
///
/// One `MintAction` covers a single policy because each policy needs its
/// own witness (native script vs Plutus script + redeemer).
#[derive(Debug, Clone)]
pub struct MintAction {
    pub policy: PolicyId,
    pub witness: MintWitness,
    /// `(name, signed-quantity)` pairs for this policy.
    pub assets: Vec<(AssetName, i128)>,
}

#[derive(Debug, Clone)]
pub enum MintWitness {
    Native(csl::NativeScript),
    Plutus {
        script: ScriptSource,
        redeemer: csl::PlutusData,
        ex_units: csl::ExUnits,
    },
}

impl MintAction {
    pub fn native(policy: PolicyId, script: csl::NativeScript) -> Self {
        Self {
            policy,
            witness: MintWitness::Native(script),
            assets: Vec::new(),
        }
    }

    pub fn plutus(
        policy: PolicyId,
        script: ScriptSource,
        redeemer: csl::PlutusData,
        ex_units: csl::ExUnits,
    ) -> Self {
        Self {
            policy,
            witness: MintWitness::Plutus {
                script,
                redeemer,
                ex_units,
            },
            assets: Vec::new(),
        }
    }

    pub fn asset(mut self, name: AssetName, qty: i128) -> Self {
        self.assets.push((name, qty));
        self
    }
}
