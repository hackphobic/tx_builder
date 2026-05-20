use cardano_serialization_lib as csl;

use crate::error::{Error, Result};

/// A 28-byte minting policy id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyId(pub [u8; 28]);

impl PolicyId {
    pub fn from_hex(s: &str) -> Result<Self> {
        let v = hex::decode(s)?;
        if v.len() != 28 {
            return Err(Error::InvalidPolicyId(s.into()));
        }
        let mut a = [0u8; 28];
        a.copy_from_slice(&v);
        Ok(PolicyId(a))
    }

    pub fn to_csl(&self) -> csl::ScriptHash {
        csl::ScriptHash::from(self.0)
    }
}

/// An asset name (0–32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetName(pub Vec<u8>);

impl AssetName {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let b = bytes.into();
        if b.len() > 32 {
            return Err(Error::InvalidAssetName(hex::encode(&b)));
        }
        Ok(AssetName(b))
    }

    pub fn from_hex(s: &str) -> Result<Self> {
        Self::new(hex::decode(s)?)
    }

    pub fn from_utf8(s: &str) -> Result<Self> {
        Self::new(s.as_bytes().to_vec())
    }

    pub fn to_csl(&self) -> Result<csl::AssetName> {
        Ok(csl::AssetName::new(self.0.clone())?)
    }
}

/// `(policy, name) -> quantity`. Quantities are signed because mint values
/// can be negative (burns).
#[derive(Debug, Clone, Default)]
pub struct Assets(pub Vec<(PolicyId, AssetName, i128)>);

impl Assets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, policy: PolicyId, name: AssetName, qty: i128) -> &mut Self {
        self.0.push((policy, name, qty));
        self
    }

    /// Convert to CSL `MultiAsset`. Returns `Ok(None)` if empty.
    /// All quantities must be non-negative; use [`Self::to_csl_mint`] for
    /// signed quantities.
    pub fn to_csl_multiasset(&self) -> Result<Option<csl::MultiAsset>> {
        if self.0.is_empty() {
            return Ok(None);
        }
        let mut ma = csl::MultiAsset::new();
        for (policy, name, qty) in &self.0 {
            if *qty < 0 {
                return Err(Error::Custom(format!(
                    "negative quantity in MultiAsset for {}.{}",
                    hex::encode(policy.0),
                    hex::encode(&name.0)
                )));
            }
            let mut assets = ma.get(&policy.to_csl()).unwrap_or_else(csl::Assets::new);
            let cur = assets.get(&name.to_csl()?).unwrap_or_else(|| csl::BigNum::from(0u64));
            let new = cur.checked_add(&csl::BigNum::from(*qty as u64))?;
            assets.insert(&name.to_csl()?, &new);
            ma.insert(&policy.to_csl(), &assets);
        }
        Ok(Some(ma))
    }
}

/// Convenience: build a CSL `Value` from lovelace and optionally a
/// MultiAsset.
pub fn csl_value(lovelace: u64, ma: Option<&csl::MultiAsset>) -> csl::Value {
    let mut v = csl::Value::new(&csl::BigNum::from(lovelace));
    if let Some(m) = ma {
        v.set_multiasset(m);
    }
    v
}
