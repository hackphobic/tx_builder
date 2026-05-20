use cardano_serialization_lib as csl;

use crate::error::Result;

/// Protocol parameters needed for transaction building.
///
/// Fetch these from your node (Blockfrost / Koios / Ogmios / `cardano-cli
/// query protocol-parameters`). They change across hard forks; the fields
/// here are aligned with what CSL's [`csl::TransactionBuilderConfigBuilder`]
/// needs plus the bits we use ourselves (cost models for the script data
/// hash, collateral params for sanity checks).
#[derive(Debug, Clone)]
pub struct ProtocolParameters {
    pub min_fee_a: u64,
    pub min_fee_b: u64,
    /// Coins-per-byte multiplier applied to *reference scripts*
    /// (Conway-era, "minfeeRefScriptCostPerByte" in protocol params).
    pub min_fee_ref_script_cost_per_byte: u64,

    pub pool_deposit: u64,
    pub key_deposit: u64,
    /// Deposit required to register a DRep (Conway-era).
    pub drep_deposit: u64,
    /// Deposit required to submit a governance proposal (Conway-era).
    pub governance_action_deposit: u64,

    pub max_tx_size: u32,
    pub max_value_size: u32,

    /// "coinsPerUtxoByte" — used for min-ada calculations.
    pub coins_per_utxo_byte: u64,

    /// (numerator, denominator)
    pub price_mem: (u64, u64),
    /// (numerator, denominator)
    pub price_step: (u64, u64),

    /// PlutusV1 / V2 / V3 cost models. Required to compute the script data
    /// hash when the transaction contains any Plutus witnesses.
    pub cost_models: csl::Costmdls,

    /// Percentage of total fee that must be put up as collateral
    /// (e.g. `150` for 150%).
    pub collateral_percentage: u32,
    pub max_collateral_inputs: u32,
}

impl ProtocolParameters {
    /// Build CSL's [`csl::TransactionBuilderConfig`].
    pub fn to_csl_config(&self) -> Result<csl::TransactionBuilderConfig> {
        let fee_algo = csl::LinearFee::new(
            &csl::BigNum::from(self.min_fee_a),
            &csl::BigNum::from(self.min_fee_b),
        );

        let ex_unit_prices = csl::ExUnitPrices::new(
            &csl::UnitInterval::new(
                &csl::BigNum::from(self.price_mem.0),
                &csl::BigNum::from(self.price_mem.1),
            ),
            &csl::UnitInterval::new(
                &csl::BigNum::from(self.price_step.0),
                &csl::BigNum::from(self.price_step.1),
            ),
        );

        // `min_fee_ref_script_cost_per_byte` is expressed in CSL as a
        // UnitInterval (rational). Cardano's actual on-chain param is a
        // single coin/byte value, so denominator = 1.
        let ref_script_coins_per_byte = csl::UnitInterval::new(
            &csl::BigNum::from(self.min_fee_ref_script_cost_per_byte),
            &csl::BigNum::from(1u64),
        );

        let cfg = csl::TransactionBuilderConfigBuilder::new()
            .fee_algo(&fee_algo)
            .pool_deposit(&csl::BigNum::from(self.pool_deposit))
            .key_deposit(&csl::BigNum::from(self.key_deposit))
            .max_value_size(self.max_value_size)
            .max_tx_size(self.max_tx_size)
            .coins_per_utxo_byte(&csl::BigNum::from(self.coins_per_utxo_byte))
            .ex_unit_prices(&ex_unit_prices)
            .ref_script_coins_per_byte(&ref_script_coins_per_byte)
            .build()?;

        Ok(cfg)
    }
}
