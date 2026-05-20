//! End-to-end test: programmatically construct addresses + UTxOs, build
//! and serialize an unsigned pubkey-only transfer, and assert it round-trips
//! through CBOR.

use cardano_tx_builder::prelude::*;

fn dummy_params() -> ProtocolParameters {
    ProtocolParameters {
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
    }
}

fn keyhash(byte: u8) -> csl::Ed25519KeyHash {
    csl::Ed25519KeyHash::from([byte; 28])
}

fn enterprise(network: u8, byte: u8) -> csl::Address {
    let cred = csl::Credential::from_keyhash(&keyhash(byte));
    csl::EnterpriseAddress::new(network, &cred).to_address()
}

#[test]
fn pubkey_transfer_round_trips() {
    let params = dummy_params();
    let wallet = enterprise(0, 0xAA);
    let dest = enterprise(0, 0xBB);

    let utxo = Utxo::new([1u8; 32], 0, wallet.clone(), 10_000_000);

    let mut b = TxBuilder::new(&params);
    b.change_address(&wallet)
        .add_pubkey_input(&utxo)
        .add_output(Output::new(&dest).lovelace(2_000_000))
        .ttl(100_000_000);

    let tx = b.build().expect("build");

    // Re-decode and confirm body shape.
    let cbor = tx.to_bytes();
    let decoded = csl::Transaction::from_bytes(cbor.clone()).expect("decode");
    assert_eq!(decoded.body().inputs().len(), 1);
    assert!(decoded.body().outputs().len() >= 1); // dest + maybe change
    assert!(decoded.body().fee().to_str().parse::<u64>().unwrap() > 0);
    assert_eq!(decoded.to_bytes(), cbor);
}

#[test]
fn requires_change_address() {
    let params = dummy_params();
    let wallet = enterprise(0, 0xAA);
    let utxo = Utxo::new([1u8; 32], 0, wallet, 10_000_000);

    let err = TxBuilder::new(&params).add_pubkey_input(&utxo).build();
    assert!(matches!(err, Err(Error::MissingChangeAddress)));
}

#[test]
fn plutus_input_requires_collateral() {
    // Use a fake plutus witness — the build should fail with MissingCollateral
    // before we ever try to validate the witness.
    let params = dummy_params();
    let wallet = enterprise(0, 0xAA);

    let script_addr = {
        let script_hash = csl::ScriptHash::from([0xCC; 28]);
        let cred = csl::Credential::from_scripthash(&script_hash);
        csl::EnterpriseAddress::new(0, &cred).to_address()
    };
    let script_utxo = Utxo::new([2u8; 32], 0, script_addr, 5_000_000);

    let dummy_script = csl::PlutusScript::new_v3(vec![0x46, 0x01, 0x01, 0x00, 0x22, 0x49]);
    let redeemer = csl::PlutusData::new_integer(&csl::BigInt::from(0u64));
    let ex_units = csl::ExUnits::new(
        &csl::BigNum::from(1_000u64),
        &csl::BigNum::from(1_000u64),
    );

    let mut b = TxBuilder::new(&params);
    b.change_address(&wallet)
        .add_input(SpendingInput::plutus(
            script_utxo,
            ScriptSource::Provided(dummy_script),
            DatumSpec::Inline,
            redeemer,
            ex_units,
        ));

    assert!(matches!(b.build(), Err(Error::MissingCollateral)));
}
