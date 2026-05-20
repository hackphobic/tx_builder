//! Example: spend a Plutus-locked UTxO and mint a token via the same
//! script, sending the unlocked ADA to a destination address.
//!
//! Run with `cargo run --example plutus_spend` *after* you fill in real
//! addresses, UTxOs, and protocol parameters. As written this won't pass
//! the validator — it just shows the shape of the API.

use tx_builder::prelude::*;
use tx_builder::value::{AssetName, Assets, PolicyId};

fn main() -> Result<()> {
    // 1. Protocol parameters (fetch these from your node / Blockfrost /
    //    Koios — these numbers are illustrative).
    let params = ProtocolParameters {
        min_fee_a: 44,
        min_fee_b: 155_381,
        min_fee_ref_script_cost_per_byte: 15,
        pool_deposit: 500_000_000,
        key_deposit: 2_000_000,
        drep_deposit: 500_000_000,
        governance_action_deposit: 100_000_000_000,
        max_tx_size: 16_384,
        max_value_size: 5_000,
        coins_per_utxo_byte: 4_310,
        price_mem: (577, 10_000),
        price_step: (721, 10_000_000),
        cost_models: csl::Costmdls::new(),
        collateral_percentage: 150,
        max_collateral_inputs: 3,
    };

    // 2. Addresses (parse from bech32 in real code).
    let wallet_addr = csl::Address::from_bech32("addr_test1...").unwrap();
    let script_addr = csl::Address::from_bech32("addr_test1...").unwrap();
    let dest_addr = csl::Address::from_bech32("addr_test1...").unwrap();

    // 3. The script UTxO we're spending (locked under Plutus).
    let script_utxo = Utxo::new(
        [0u8; 32],
        0,
        script_addr.clone(),
        5_000_000,
    )
    .with_inline_datum(csl::PlutusData::new_integer(&csl::BigInt::from(42u64)));

    // 4. A wallet UTxO for fees + collateral.
    let wallet_utxo = Utxo::new([1u8; 32], 0, wallet_addr.clone(), 10_000_000);

    // 5. A reference UTxO holding the Plutus script (CIP-33).
    let ref_script_utxo = Utxo::new([2u8; 32], 0, script_addr.clone(), 1_500_000);
    let script_hash = csl::ScriptHash::from([0u8; 28]);

    // 6. Redeemer and ex-units (replace with real evaluator output).
    let redeemer = csl::PlutusData::new_integer(&csl::BigInt::from(0u64));
    let ex_units = csl::ExUnits::new(
        &csl::BigNum::from(2_000_000u64),
        &csl::BigNum::from(800_000_000u64),
    );

    let script_source = ScriptSource::Reference {
        utxo: Box::new(ref_script_utxo),
        script_hash: script_hash.clone(),
        language: csl::Language::new_plutus_v3(),
        script_size: 1500,
    };

    // 7. Mint action driven by the same script (1 NFT).
    let policy = PolicyId(script_hash.to_bytes().try_into().unwrap());
    let mint = MintAction::plutus(
        policy,
        script_source.clone(),
        redeemer.clone(),
        ex_units.clone(),
    )
    .asset(AssetName::from_utf8("Receipt")?, 1);

    // 8. Assemble.
    let mut b = TxBuilder::new(&params);
    b.change_address(&wallet_addr)
        .add_pubkey_input(&wallet_utxo)
        .add_input(SpendingInput::plutus(
            script_utxo,
            script_source,
            DatumSpec::Inline,
            redeemer,
            ex_units,
        ))
        .add_collateral(&wallet_utxo)
        .total_collateral(5_000_000)
        .add_mint(mint)
        .add_output(
            Output::new(&dest_addr)
                .lovelace(4_000_000)
                .assets({
                    let mut a = Assets::new();
                    a.add(policy, AssetName::from_utf8("Receipt")?, 1);
                    a
                }),
        )
        .ttl(99_999_999);

    let tx = b.build()?;

    let cbor = hex::encode(tx.to_bytes());
    println!("unsigned tx (cbor hex, {} bytes):\n{}", cbor.len() / 2, cbor);

    Ok(())
}
